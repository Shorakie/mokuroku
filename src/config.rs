use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serenity::model::id::ApplicationId;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Discord {
    pub token: String,
    pub bot_id: Option<ApplicationId>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub discord: Discord,
    pub command_prefix: String,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or("development".into());
        Config::builder()
            .add_source(File::with_name("config/base"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(
                Environment::with_prefix("MR")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .set_default("command_prefix", "mr!")?
            .build()?
            .try_deserialize()
    }
}
