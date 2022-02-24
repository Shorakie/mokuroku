/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
mod commands;

use std::{collections::HashSet, env, sync::Arc};

use commands::{help::*};
use serenity::{
    async_trait,
    client::{bridge::gateway::ShardManager, validate_token},
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use tracing::{error, info};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}!", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(help)]
struct General;

#[tokio::main]
async fn main() {
    // load .env file
    dotenv::dotenv().expect("Failed to load .env file");

    // reads RUST_LOG env
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // ensure token is valid
    assert!(validate_token(&token).is_ok());

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(
            |c| c.owners(owners)
            .prefix("mr!")
        )
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    // listen for Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register Ctrl+C handler");
        info!("Shuting down all shards...");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}