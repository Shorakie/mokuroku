use std::sync::Arc;

use mongodm::prelude::MongoClient;
use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::{Mutex, TypeMapKey},
};

pub struct ShardManagerContainer;
pub struct DatabaseContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for DatabaseContainer {
    type Value = MongoClient;
}
