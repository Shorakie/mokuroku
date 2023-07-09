use crate::events::Handler;
use std::collections::HashSet;

use config::ConfigError;
use eyre::{bail, eyre, Result};
use serenity::{
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::id::{ApplicationId, UserId},
    prelude::{Client as SerenityClient, GatewayIntents},
};
use tracing::{info, warn};

use crate::commands::media::*;
use crate::config::Configuration;

#[group]
#[commands(anime)]
struct Media;

pub struct Client {
    client: SerenityClient,
}

impl Client {
    pub async fn new() -> Result<Client> {
        let config = match Configuration::new() {
            Ok(config) => config,
            Err(ConfigError::NotFound(variable)) => {
                bail!("⚠️Missing environment variable: {variable}.")
            }
            _ => bail!("Failed to load environment variables."),
        };

        // Set gateway intents, which decides what events the bot will be notified about
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

        // We will fetch your bot's owners and id
        let (owners, bot_id) = Self::get_bot_info(&config).await?;
        info!(
            "Bot information:
              Bot Id: {}
              Bot Owner(s): {}",
            bot_id,
            owners
                .iter()
                .map(|o| o.0.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let framework = StandardFramework::new()
            .configure(|c| c.owners(owners).prefix(&config.command_prefix))
            .group(&MEDIA_GROUP);

        let client = SerenityClient::builder(&config.discord.token, intents)
            .framework(framework)
            .event_handler(Handler)
            .await?;

        Ok(Client { client })
    }

    pub async fn start(&mut self) -> Result<()> {
        let shard_manager = self.client.shard_manager.clone();

        // listen for Ctrl+C
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Could not register Ctrl+C handler");
            info!("Shuting down all shards...");
            shard_manager.lock().await.shutdown_all().await;
        });

        self.client.start().await?;

        Ok(())
    }

    pub async fn get_bot_info(config: &Configuration) -> Result<(HashSet<UserId>, ApplicationId)> {
        let http = Http::new(&config.discord.token);
        match http.get_current_application_info().await {
            Ok(info) => {
                let mut owners = HashSet::new();

                owners.insert(info.owner.id);

                // include application team members as owners
                if let Some(team) = info.team {
                    for member in &team.members {
                        owners.insert(member.user.id);
                    }
                }

                Ok((owners, info.id))
            }
            Err(why) => {
                warn!("Could not access application info: {:?}", why);
                warn!("trying environment variable for bot id");

                let bot_id = config
                    .discord
                    .bot_id
                    .ok_or_else(|| eyre!("Unable to find DISCORD_BOT_ID environment variable."))?;
                Ok((HashSet::new(), bot_id))
            }
        }
    }
}
