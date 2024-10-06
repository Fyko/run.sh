use std::error::Error;
use std::sync::Arc;

use dotenvy::dotenv;
use futures::{SinkExt, StreamExt};
use run_sh::{
    config::CONFIG,
    hypervisor::{languages::Languages, Hypervisor},
};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let hypervisor = Arc::new(Hypervisor::new(CONFIG.docker_endpoint.clone()));
    hypervisor.init().await?;

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    tracing::info!("tcp listening on 127.0.0.1:8080");

    // loop {
    //     let (socket, addr) = listener.accept().await?;
    //     tokio::spawn({
    //         let docker = docker.clone();
    //         async move {
    //             handle_connection(docker, socket, addr).await;
    //         }
    //     });
    // }
    tokio::select! {
        _ = shutdown_signal() => {
            hypervisor.stop().await?;
        },
        _ = run_server(hypervisor.clone(), listener) => {},
    }

    Ok(())
}

async fn run_server(hypervisor: Arc<Hypervisor>, listener: TcpListener) {
    loop {
        while let Ok((socket, addr)) = listener.accept().await {
            tokio::spawn({
                let hypervisor = hypervisor.clone();
                async move {
                    handle_connection(hypervisor, socket, addr).await;
                }
            });
        }
    }
}

async fn handle_connection(hypervisor: Arc<Hypervisor>, socket: TcpStream, addr: SocketAddr) {
    let mut lines = Framed::new(socket, LinesCodec::new());

    tracing::info!("new connection from {addr}");
    let langs = CONFIG
        .languages
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    lines
        .send(indoc::formatdoc! {"
			Hello! This is the test server.

			Enabled languages: {langs}
			To add more, edit your `LANGUAGES` environment variable.

			To execute code, send the code in the following format:
			<language> <code>
		"})
        .await
        .expect("failed to write data to socket");
    loop {
        let input = lines.next().await;
        match input {
            Some(Ok(line)) => {
                let (language, code) = line.split_once(' ').unwrap();
                let language = language.trim();
                let code = code.trim();
                tracing::info!("{language:#?} received code: {code:#?}");
                let Some(language) = Languages::from_codeblock_language(language) else {
                    lines
                        .send("Invalid language")
                        .await
                        .expect("failed to write data to socket");
                    continue;
                };

                if !language.enabled() {
                    tracing::warn!("language {language} is not enabled");
                    lines
                        .send("Language not enabled")
                        .await
                        .expect("failed to write data to socket");
                }

                lines
                    .send("Executing code...")
                    .await
                    .expect("failed to write data to socket");
                // execute code
                let res = hypervisor.exec(&language, code).await;
                match res {
                    Ok(output) => {
                        let out = output
                            .iter()
                            .map(|b| String::from_utf8_lossy(b))
                            .collect::<Vec<_>>()
                            .join("\n");
                        lines
                            .send(out)
                            .await
                            .expect("failed to write data to socket");
                    }
                    Err(e) => {
                        lines
                            .send(format!("Error: {e:#?}"))
                            .await
                            .expect("failed to write data to socket");
                    }
                }
            }
            // An error occurred.
            Some(Err(e)) => {
                tracing::error!("an error occurred while processing input {e:#?}",);
            }
            // The stream has been exhausted.
            None => break,
        }
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
