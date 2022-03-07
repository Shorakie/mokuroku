use std::{collections::HashSet, env, sync::Arc};

use commands::{help::*};
use serenity::{
    async_trait,
    Client,
    client::{
        bridge::gateway::ShardManager,
        validate_token,
        EventHandler
    },
    framework::{
        standard::macros::{
            group,
            hook,
        },
        StandardFramework
    },
    http::Http,
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::Ready,
    },
    prelude::{
        Context,
        Mutex,
        TypeMapKey,
    },
};
use tracing::{error, info, debug, instrument};


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

    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

#[group]
#[commands(help)]
struct General;

#[hook]
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!("Got command '{}' by user '{}'", command_name, msg.author.name);
    true
}

#[tokio::main]
#[instrument]
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
            .prefix("mr~")
        )
        .before(before)
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