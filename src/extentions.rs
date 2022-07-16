use std::fmt::Display;

use crate::{
    config::Config,
    embeds::{make_blank_embed, make_error_embed, make_success_embed},
    utils::{ConfigContainer, MongoContainer},
};

use anyhow::{Context, Result};
use mongodm::prelude::MongoDatabase;
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client,
    model::{
        channel::Message,
        id::ChannelId,
        interactions::{message_component::MessageComponentInteraction, InteractionResponseType},
    },
};

#[async_trait]
pub trait ClientContextExt {
    async fn get_config(&self) -> Config;
    async fn get_db(&self) -> MongoDatabase;
}

#[async_trait]
impl ClientContextExt for client::Context {
    async fn get_config(&self) -> Config {
        let data = self.data.read().await;
        data.get::<ConfigContainer>().unwrap().clone()
    }

    async fn get_db(&self) -> MongoDatabase {
        let data = self.data.read().await;
        let config = data.get::<ConfigContainer>().unwrap().clone();
        data.get::<MongoContainer>()
            .unwrap()
            .database(config.mongo_database.as_str())
    }
}

#[async_trait]
pub trait ChannelIdExt {
    async fn send_embed<F>(&self, ctx: &client::Context, build: F) -> Result<Message>
    where
        F: FnOnce(&mut CreateEmbed) + Sync + Send;
}

#[async_trait]
impl ChannelIdExt for ChannelId {
    async fn send_embed<F>(&self, ctx: &client::Context, build: F) -> Result<Message>
    where
        F: FnOnce(&mut CreateEmbed) + Sync + Send,
    {
        let embed = make_blank_embed(|e| {
            build(e);
            e
        });
        self.send_message(&ctx, |m| m.set_embed(embed))
            .await
            .context("Failed to send embed message")
    }
}

#[async_trait]
pub trait MessageComponentInteractionExt {
    async fn deferred_ephemeral(&self, ctx: &client::Context) -> Result<()>;

    async fn reply_error(
        &self,
        ctx: &client::Context,
        s: impl Display + Send + Sync + 'static,
    ) -> Result<()>;

    async fn reply_success(
        &self,
        ctx: &client::Context,
        s: impl Display + Send + Sync + 'static,
        title: impl Display + Send + Sync + 'static,
    ) -> Result<()>;

    async fn ack(&self, ctx: &client::Context) -> Result<()>;
}

#[async_trait]
impl MessageComponentInteractionExt for MessageComponentInteraction {
    async fn deferred_ephemeral(&self, ctx: &client::Context) -> Result<()> {
        self.create_interaction_response(&ctx, |resp| {
            resp.kind(InteractionResponseType::DeferredChannelMessageWithSource);
            resp.interaction_response_data(|data| data.ephemeral(true))
        })
        .await
        .context("Failed to send deffered interaction ephemeral reply")
    }

    async fn reply_error(
        &self,
        ctx: &client::Context,
        s: impl Display + Send + Sync + 'static,
    ) -> Result<()> {
        let embed = make_error_embed(|e| e.description(s));
        self.create_interaction_response(&ctx, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource);
            resp.interaction_response_data(|data| {
                data.ephemeral(true);
                data.set_embed(embed)
            })
        })
        .await
        .context("Failed to send interaction error reply")
    }

    async fn reply_success(
        &self,
        ctx: &client::Context,
        s: impl Display + Send + Sync + 'static,
        title: impl Display + Send + Sync + 'static,
    ) -> Result<()> {
        let embed = make_success_embed(|e| e.description(s).title(title));

        self.create_interaction_response(&ctx, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource);
            resp.interaction_response_data(|data| {
                data.ephemeral(true);
                data.set_embed(embed.clone())
            })
        })
        .await
        .context("Failed to send interaction error reply")
    }

    async fn ack(&self, ctx: &client::Context) -> Result<()> {
        self.create_interaction_response(&ctx, |resp| {
            resp.kind(InteractionResponseType::DeferredUpdateMessage)
        })
        .await
        .context("Failed to send interaction ack")
    }
}
