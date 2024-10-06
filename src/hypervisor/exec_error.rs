use docker_api::Error as DockerError;

#[derive(thiserror::Error, Debug)]
pub enum ExecError {
    #[error("code execution timed out")]
    Timeout,

    #[error("no output")]
    Empty,

    #[error("an error occurred with docker")]
    Docker(#[from] DockerError),

    #[error("an error occurred with the docker connection")]
    DockerConnection(#[from] docker_api::conn::Error),
}
