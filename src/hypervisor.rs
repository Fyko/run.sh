use docker_api::{
    conn::TtyChunk,
    opts::{
        ContainerCreateOpts, ContainerFilter, ContainerListOpts, ExecCreateOpts, ImageBuildOpts,
    },
    Container, Docker as DockerClient,
};
use exec_error::ExecError;
use futures::StreamExt;
use languages::Languages;
use rand::Rng;
use std::str;
use std::{env, sync::Arc};

use crate::config::CONFIG;

pub mod exec_error;
pub mod languages;

macro_rules! exec_options {
    ($command:expr, $($arg:expr),*) => {
        ExecCreateOpts::builder()
            .command(&[$command, $($arg),*])
            .attach_stdout(true)
            .attach_stderr(true)
            .build()
    };
}

pub struct Hypervisor {
    client: Arc<DockerClient>,
}

impl Hypervisor {
    pub fn new(endpoint: String) -> Self {
        let client = Arc::new(DockerClient::new(endpoint).expect("failed to create docker client"));

        Self { client }
    }

    pub async fn init(&self) -> docker_api::errors::Result<()> {
        for language in &CONFIG.languages {
            tracing::info!("building image for {language}");
            self.build_image(language).await?;
            // tracing::info!("running container for {language}");
            // self.run(language).await?;
        }

        Ok(())
    }

    pub async fn stop(&self) -> docker_api::errors::Result<()> {
        let opts = ContainerListOpts::builder()
            .all(true)
            .filter([ContainerFilter::Name("run.sh_*".to_string())])
            .build();

        let containers = self.client.containers().list(&opts).await?;
        tracing::debug!("killing {} containers", containers.len());
        for container in containers {
            let names = container.names.unwrap(); // this is infallible thanks to our names filter
            let name = names.first().unwrap().to_owned();
            let name = if let Some(stripped) = name.strip_prefix("/") {
                stripped.to_owned()
            } else {
                name
            };

            tracing::debug!("killing container {name}");
            tokio::spawn({
                let client = self.client.clone();
                async move {
                    let container = client.containers().get(name);
                    container
                        .kill(None)
                        .await
                        .expect("failed to kill container");
                }
            });
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn build_image(&self, language: &Languages) -> docker_api::errors::Result<()> {
        let cwd = env::current_dir()?;
        let cwd = cwd.display();

        let opts = ImageBuildOpts::builder(format!("{cwd}/languages/{language}"))
            .tag(format!("run.sh_{language}:latest"))
            .build();

        let images = self.client.images();
        let mut stream = images.build(&opts);
        while (stream.next().await).is_some() {}

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn run(
        &self,
        language: &Languages,
        id: u32,
    ) -> docker_api::errors::Result<Container> {
        let container = self.create_container(language, id).await?;
        self.start_container(language, id).await?;

        Ok(container)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn start_container(
        &self,
        language: &Languages,
        id: u32,
    ) -> docker_api::errors::Result<()> {
        let container_name = format!("run.sh_{language}_{id}");
        let container = self.client.containers().get(container_name);
        let _ = container.start().await;

        tracing::debug!("creating /tmp/eval directory");
        let options = exec_options!("mkdir", "-p", "/tmp/eval");
        let mut stream = container.exec(&options, &Default::default()).await?;
        if let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tracing::debug!("stdout: {chunk:#?}");
        }

        tracing::debug!("chmoding folder");
        let options = exec_options!("chmod", "771", "/tmp/eval");
        let mut stream = container.exec(&options, &Default::default()).await?;
        if let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tracing::debug!("stdout: {chunk:#?}");
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn create_container(
        &self,
        language: &Languages,
        id: u32,
    ) -> Result<docker_api::Container, docker_api::Error> {
        let opts = ContainerCreateOpts::builder()
            .name(format!("run.sh_{language}_{id}"))
            .auto_remove(true)
            .user("1000:1000")
            .working_dir("/tmp")
            .network_mode("none")
            .tty(true)
            .cpus(0.25)
            .memory(128 * 1024 * 1024)
            .memory_swap(128 * 1024 * 1024)
            .runtime(CONFIG.docker_runtime.clone())
            .image(format!("run.sh_{language}:latest"))
            .command(["tail", "-f", "/dev/null"]);

        let opts = opts.build();

        tracing::debug!("creating container");
        self.client.containers().create(&opts).await
    }

    #[tracing::instrument(level = "debug", skip(self, code))]
    pub async fn exec(&self, language: &Languages, code: &str) -> Result<Vec<Vec<u8>>, ExecError> {
        let id = rand::thread_rng().gen_range(u32::MIN..u32::MAX);
        let dir = format!("/tmp/eval/{id}");
        let container_name = format!("run.sh_{language}_{id}");
        tracing::debug!("container name: {container_name}");
        self.run(language, id).await?;
        let container = self.client.containers().get(&container_name);

        tracing::debug!("creating unique folder in container");
        let mut stream = container
            .exec(&exec_options!("mkdir", "-p", &dir), &Default::default())
            .await?;
        if let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tracing::debug!("stdout: {chunk:#?}");
        }

        tracing::debug!("chmoding folder");
        let mut stream = container
            .exec(&exec_options!("chmod", "777", &dir), &Default::default())
            .await?;
        while (stream.next().await).is_some() {}

        // execute code in container
        tracing::debug!("executing code in container");
        let options = ExecCreateOpts::builder()
            .command(["/bin/sh", "/var/run/run.sh", code])
            .user("1001:1001")
            .working_dir(dir)
            .attach_stdout(true)
            .attach_stderr(true)
            .build();

        let mut stream = container.exec(&options, &Default::default()).await?;
        let mut res = vec![];
        let timeout = tokio::time::sleep(std::time::Duration::from_secs(10));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = &mut timeout => return Err(ExecError::Timeout),
                chunk = stream.next() => match chunk {
                    Some(Ok(TtyChunk::StdOut(bytes) | TtyChunk::StdErr(bytes))) => {
                        res.push(bytes);
                    },
                    Some(Ok(TtyChunk::StdIn(_))) => unreachable!(),
                    Some(Err(e)) => {
                        // kill container
                        tracing::debug!("killing container");
                        container.kill(None).await?;

                        return Err(ExecError::DockerConnection(e));
                    },
                    None => break,
                }
            }
        }

        // kill container
        tracing::debug!("killing container");
        container.kill(None).await?;

        Ok(res)
    }
}

/// Formats the output of a code execution for Discord.
///
/// - Applies a truncation of 1500 characters if the output is longer than that.
/// - If the output is empty, returns "No output".
pub fn format_output(code_result: Vec<Vec<u8>>) -> String {
    let out = code_result
        .iter()
        .map(|b| String::from_utf8_lossy(b))
        .collect::<Vec<_>>()
        .join("\n");

    let out = if out.len() > 1500 {
        let trunc = out.len() - 1500;
        let mut out = out;
        out.truncate(1500);
        out.push_str(&format!("...({trunc} more characters)"));

        out
    } else {
        out
    };

    if out.is_empty() {
        "No output".to_string()
    } else {
        out
    }
}
