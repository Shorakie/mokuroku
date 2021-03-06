pub mod commands;
pub mod db;
pub mod embeds;
pub mod extentions;
pub mod graphql;
pub mod paginator;
pub mod strings;
pub mod tests;
pub mod utils;

use crate::{
    commands::{anime::lookup::*, help::*},
    db::watchlist::WatchInfoCollConf,
    utils::{DatabaseContainer, ShardManagerContainer},
};

use mongodm::{
    prelude::{MongoClient, MongoClientOptions},
    sync_indexes,
};
use serenity::{
    async_trait,
    client::EventHandler,
    framework::{
        standard::{
            macros::{group, hook},
            CommandError,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready},
    prelude::{Context, GatewayIntents},
    utils::validate_token,
    Client,
};
use std::{collections::HashSet, env};
use tracing::{debug, error, info, instrument};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}!", ready.user.name);
    }

    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

#[group]
#[commands(help, lookup)]
struct General;

#[hook]
#[instrument]
async fn after(_: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        error!(
            "Error in {} invoked by {} from {}@{}: {:?}",
            cmd_name,
            msg.author.tag(),
            msg.channel_id,
            msg.guild_id
                .map_or("private".to_owned(), |id| id.0.to_string()),
            why
        );
    }
}

#[hook]
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name,
        msg.author.tag()
    );
    true
}

#[tokio::main]
#[instrument]
async fn main() {
    // load .env file
    dotenv::dotenv().expect("Failed to load .env file");

    // reads RUST_LOG env
    tracing_subscriber::fmt::init();

    let mongo_uri = env::var("MONGODB_URI").expect("Expected a MONGO_URI in the environment");
    let mongo_database = env::var("MONGODB_NAME")
        .as_deref()
        .unwrap_or("mokuroku")
        .to_owned();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let prefix = env::var("BOT_PREFIX")
        .as_deref()
        .unwrap_or("mr~")
        .to_owned();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    // ensure token is valid
    assert!(validate_token(&token).is_ok());

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(prefix))
        .before(before)
        .after(after)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // initiate mongo client
    let mongo_options = MongoClientOptions::parse(mongo_uri)
        .await
        .expect("Couldn't parse the Mongo URI");
    let mongo =
        MongoClient::with_options(mongo_options).expect("Couldn't instantiate mongo client");

    // sync mongo indexes
    sync_indexes::<WatchInfoCollConf>(&mongo.database(&mongo_database))
        .await
        .expect("Can not sync indexes for Watchinfo collection");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<DatabaseContainer>(mongo.clone());
    }

    let shard_manager = client.shard_manager.clone();

    // listen for Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register Ctrl+C handler");
        info!("Shuting down all shards...");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
