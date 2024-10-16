use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use dotenvy::dotenv;
use run_sh::config::CONFIG;
use run_sh::events::{interaction_create, message_create, message_update, thread_create};
use run_sh::hypervisor::Hypervisor;
use run_sh::state::BotState;
use run_sh::{commands, BotFramework};
use sqlx::postgres::PgPoolOptions;
use tokio::task::JoinSet;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use twilight_gateway::{
    create_recommended, CloseFrame, Config, Event, EventTypeFlags, Intents, Shard, StreamExt,
};
use twilight_http::Client;
use vesper::framework::Framework;

static READY: AtomicBool = AtomicBool::new(false);
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "systemd")]
    systemd();
    dotenv().ok();

    Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sqlx::query=debug,run_sh=debug,info".into()),
        )
        .with(fmt::layer())
        .init();
    tracing::info!(env = CONFIG.environment.to_string(), "starting up");

    let discord_client = Arc::new(Client::new(CONFIG.discord_token.clone()));
    let hypervisor = Arc::new(Hypervisor::new(CONFIG.docker_endpoint.clone()));

    tracing::debug!("connecting to database {}", CONFIG.database_url);
    let db = PgPoolOptions::new()
        .max_connections(100)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(60))
        .connect(&CONFIG.database_url)
        .await?;

    sqlx::migrate!().run(&db).await?;
    let count = sqlx::query!("select count(*) as count from execution")
        .fetch_one(&db)
        .await?
        .count
        .unwrap_or(0);

    tracing::info!("initialized database with {count} executions");

    let state = BotState { hypervisor, db };

    tracing::info!("initializing docker containers");
    state.hypervisor.init().await?;

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
    let shards = create_recommended(&discord_client, config, |_, builder| builder.build())
        .await?
        .collect::<Vec<_>>();
    let mut senders = Vec::with_capacity(shards.len());
    let mut tasks = JoinSet::new();

    for shard in shards {
        senders.push(shard.sender());
        tasks.spawn(runner(shard, framework.clone()));
    }

    READY.store(true, Ordering::Relaxed);
    shutdown_signal().await;
    SHUTDOWN.store(true, Ordering::Relaxed);
    for sender in senders {
        _ = sender.close(CloseFrame::NORMAL);
    }

    tasks.join_all().await;

    framework.data.hypervisor.stop().await?;
    framework.data.db.close().await;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all, fields(shard = shard.id().number()))]
async fn runner(mut shard: Shard, framework: BotFramework) {
    let shard_id = shard.id().number();

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => {
                tracing::info!("shutting down shard {shard_id}");
                break;
            }
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "error receiving event");

                continue;
            }
        };

        tokio::spawn({
            let framework = framework.clone();
            async move {
                match event {
                    Event::ThreadCreate(event) => {
                        match thread_create::handle(framework.clone(), event).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("failed to handle thread create event - {e:#?}")
                            }
                        }
                    }
                    Event::InteractionCreate(event) => {
                        match interaction_create::handle(framework.clone(), event).await {
                            Ok(_) => {}
                            Err(e) => tracing::error!(
                                "failed to handle interaction create event - {e:#?}"
                            ),
                        }
                    }
                    Event::MessageCreate(event) => {
                        match message_create::handle(framework.clone(), event).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("failed to handle message create event - {e:#?}")
                            }
                        }
                    }
                    Event::MessageUpdate(event) => {
                        match message_update::handle(framework.clone(), event).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("failed to handle message update event - {e:#?}")
                            }
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
        });
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

#[cfg(feature = "systemd")]
fn systemd() {
    use libsystemd::daemon::{self, NotifyState};
    tokio::spawn(async move {
        if libsystemd::daemon::booted() {
            tokio::spawn(async move {
                while !READY.load(Ordering::Relaxed) {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }

                let sent =
                    daemon::notify(false, &[NotifyState::Ready]).expect("notifying systemd failed");
                if !sent {
                    tracing::warn!("failed to notify systemd");
                }
            });

            if let Some(timeout) = daemon::watchdog_enabled(true) {
                tracing::info!("watchdog enabled");
                loop {
                    daemon::notify(false, &[NotifyState::Watchdog])
                        .expect("notifying systemd failed");
                    tokio::time::sleep(timeout / 2).await;
                }
            } else {
                tracing::warn!("watchdog not enabled");
            }
        } else {
            tracing::warn!("systemd not booted");
        }
    });
}
