use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use dotenvy::dotenv;
use run_sh::config::CONFIG;
use run_sh::events::{interaction_create, message_create, message_update, thread_create};
use run_sh::hypervisor::Docker;
use run_sh::state::BotState;
use run_sh::{commands, BotFramework};
use sqlx::SqlitePool;
use tokio::task::JoinSet;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Shard;
use twilight_gateway::{
    stream::{self},
    CloseFrame, Config, Event, Intents,
};
use twilight_http::Client;
use vesper::framework::Framework;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sqlx::query=debug,run_sh=debug,info".into()),
        )
        .with(fmt::layer())
        .init();
    tracing::info!(env = CONFIG.environment.to_string(), "starting up");

    let cache = Arc::new(InMemoryCache::new());
    let discord_client = Arc::new(Client::new(CONFIG.discord_token.clone()));
    let docker = Arc::new(Docker::new(CONFIG.docker_endpoint.clone()));

    let db = SqlitePool::connect(&CONFIG.database_url).await?;
    sqlx::migrate!().run(&db).await?;
    let count = sqlx::query!("select count(*) as count from execution")
        .fetch_one(&db)
        .await?
        .count;

    tracing::info!("initialized database with {count} executions");

    let state = BotState { cache, docker, db };

    tracing::info!("initializing docker containers");
    state.docker.init().await?;

    let framework = Arc::new(
        Framework::builder(discord_client.clone(), CONFIG.discord_application_id, state)
            .command(commands::execute_code::execute_code)
            .command(commands::languages::languages)
            .build(),
    );

    let config = Config::new(
        CONFIG.discord_token.clone(),
        Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    );
    let shards = stream::create_recommended(&discord_client, config, |_, builder| builder.build())
        .await?
        .collect::<Vec<_>>();
    let mut senders = Vec::with_capacity(shards.len());
    let mut tasks = JoinSet::new();

    for shard in shards {
        senders.push(shard.sender());
        tasks.spawn(runner(shard, framework.clone()));
    }

    #[cfg(feature = "systemd")]
    {
        use libsystemd::daemon::{self, NotifyState};
        if libsystemd::daemon::booted() {
            let sent =
                daemon::notify(true, &[NotifyState::Ready]).expect("notifying systemd failed");
            if !sent {
                tracing::warn!("failed to notify systemd");
            }
        }
        {
            tracing::warn!("systemd not booted");
        }
    }

    shutdown_signal().await;
    SHUTDOWN.store(true, Ordering::Relaxed);
    for sender in senders {
        _ = sender.close(CloseFrame::NORMAL);
    }

    while (tasks.join_next().await).is_some() {}
    framework.data.docker.stop().await?;
    framework.data.db.close().await;

    Ok(())
}

async fn runner(mut shard: Shard, framework: BotFramework) {
    let shard_id = shard.id().number();

    while let Ok(event) = shard.next_event().await {
        framework.data.cache.update(&event);

        match event {
            Event::GatewayClose(_) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Event::ThreadCreate(event) => {
                match thread_create::handle(framework.clone(), event).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("failed to handle thread create event - {e:#?}"),
                }
            }
            Event::InteractionCreate(event) => {
                match interaction_create::handle(framework.clone(), event).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("failed to handle interaction create event - {e:#?}"),
                }
            }

            Event::MessageCreate(event) => {
                match message_create::handle(framework.clone(), event).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("failed to handle message create event - {e:#?}"),
                }
            }

            Event::MessageUpdate(event) => {
                match message_update::handle(framework.clone(), event).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("failed to handle message update event - {e:#?}"),
                }
            }
            Event::Ready(ready) => {
                let name = ready.user.name;
                tracing::info!("shard {shard_id} connected; {name} ready!");
            }
            Event::GatewayReconnect => tracing::info! {
                target: "gateway_reconnect",
                "shard {shard_id} gateway reconnecting"
            },
            _ => {}
        };
    }
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
