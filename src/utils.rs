use std::{env, sync::Arc};

use anyhow::{bail, Result};
use mongodm::prelude::MongoClient;
use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::{Mutex, TypeMapKey},
};

use crate::config::Config;

pub struct ShardManagerContainer;
pub struct MongoContainer;
pub struct ConfigContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for MongoContainer {
    type Value = MongoClient;
}

impl TypeMapKey for ConfigContainer {
    type Value = Config;
}

pub fn get_env(key: &str, default: Option<String>) -> Result<String> {
    match env::var(key) {
        Ok(s) => Ok(s),
        Err(_) => {
            if let Some(def) = default {
                Ok(def)
            } else {
                bail!("missing environment variable {}", key);
            }
        }
    }
}
