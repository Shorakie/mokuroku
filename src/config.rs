use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::utils::get_env;

pub const ANILIST_API: &str = "https://graphql.anilist.co/";

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub command_prefix: String,

    pub mongo_uri: String,
    pub mongo_database: String,

    pub start_time: DateTime<Utc>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            discord_token: get_env("DISCORD_TOKEN", None)?,
            command_prefix: get_env("COMMAND_PREFIX", Some("mr~".to_owned()))?,
            mongo_uri: get_env("MONGODB_URI", None)?,
            mongo_database: get_env("MONGODB_DATABASE", Some("mokuroku".to_owned()))?,
            start_time: Utc::now(),
        })
    }
}
