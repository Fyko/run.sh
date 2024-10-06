use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::ApplicationMarker, Id};

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().expect("Unable to retrieve config"));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Environment {
    /// Development environment
    #[serde(rename = "development")]
    Development,
    /// Production environment
    #[serde(rename = "production")]
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Environment::Development => "development",
            Environment::Production => "production",
        };
        write!(f, "{out}")
    }
}

/// Application Config
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The environment the application is running in
    pub environment: Environment,

    /// The Discord token for the bot
    pub discord_token: String,

    /// The Discord application id for the bot
    pub discord_application_id: Id<ApplicationMarker>,

    /// The enabled languages
    pub languages: Vec<String>, // todo: use the enum

    // The Docker api endpoint
    pub docker_endpoint: String,

    /// The database url
    pub database_url: String,
}

impl Config {
    /// Create a new `Config`
    pub fn new() -> Result<Self> {
        let config = envy::from_env::<Self>()?;

        Ok(config)
    }
}

/// Get the default static `Config`
pub fn get_config() -> &'static Config {
    &CONFIG
}
