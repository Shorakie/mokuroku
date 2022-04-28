use std::time::Duration;

use anyhow::anyhow;
use mongodm::ToRepository;
use serenity::{
    builder::CreateComponents,
    framework::standard::{macros::command, Args, CommandResult},
    futures::StreamExt,
    model::{channel::Message, interactions::InteractionResponseType},
    prelude::Context,
};

use crate::{
    db::watchlist::{WatchInfo, WatchListCollectionExt},
    extentions::{ClientContextExt, MessageComponentInteractionExt},
    graphql::lookup_media_page::MediaType,
    paginator::{AsComponent, AsEmbed, EmbedPaginator, MediaPaginator},
};

#[command("anime")]
#[usage("anime <anime name>")]
#[min_args(1)]
pub async fn lookup(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // validate arguments
    let anime_name = args.trimmed().quoted().rest();

    // query anime page 1
    let mut media_paginator = MediaPaginator::new(anime_name.to_owned(), MediaType::Anime).await?;

    let mut current_media = match media_paginator.current_item() {
        Some(media) => media,
        None => {
            msg.reply(&ctx.http, "Cannot find any anime...").await?;
            return Ok(());
        }
    };

    // send anime card
    let mut anime_card = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.set_embed(current_media.as_embed())
                .set_components(media_paginator.as_component())
        })
        .await?;

    let watch_info_repo = ctx.get_db().await.repository::<WatchInfo>();
    let template_parser = liquid::ParserBuilder::with_stdlib().build()?;

    // listen for component interactions
    let mut interaction_collector = anime_card
        .await_component_interactions(&ctx)
        .timeout(Duration::from_secs(60))
        .build();
    while let Some(interaction) = interaction_collector.next().await {
        let mut next_media = None;
        let (mut watch_info, mut template) = (None, None);

        match interaction.data.custom_id.as_str() {
            "PREV_PAGE" => next_media = media_paginator.prev_item().await,
            "NEXT_PAGE" => next_media = media_paginator.next_item().await,
            "WATCH" => {
                (watch_info, template) = (
                    watch_info_repo
                        .toggle_consuming(&current_media, msg.author.id)
                        .await
                        .transpose(),
                    Some(template_parser.parse("You {{ status_verb }} _it_ {{ status_emoji }}")?),
                )
            }
            "FINISH" => {
                (watch_info, template) = (
                    watch_info_repo
                        .toggle_finish(&current_media, msg.author.id)
                        .await
                        .transpose(),
                    Some(template_parser.parse("You {{ status_verb }} _it_ {{ status_emoji }}")?),
                )
            }
            "SUGGEST" => {
                (watch_info, template) = (
                    watch_info_repo
                        .toggle_suggestion(&current_media, msg.author.id)
                        .await
                        .transpose(),
                    Some(template_parser.parse(
                        "You are {{ suggestion_verb }} suggesting _it_ {{ suggestion_emoji }}",
                    )?),
                )
            }
            _ => return Ok(()),
        }

        match interaction.data.custom_id.as_str() {
            // Update message on page change
            "NEXT_PAGE" | "PREV_PAGE" => match next_media {
                Some(media) => {
                    current_media = media;
                    interaction
                        .create_interaction_response(&ctx, |resp| {
                            resp.kind(InteractionResponseType::UpdateMessage)
                                .interaction_response_data(|data| {
                                    data.set_embed(current_media.as_embed())
                                        .set_components(media_paginator.as_component())
                                })
                        })
                        .await?;
                }
                None => interaction.ack(ctx).await?,
            },
            // Send correct reply message
            "WATCH" | "FINISH" | "SUGGEST" => match watch_info {
                Some(Ok(info)) => {
                    if let Some(template) = template {
                        let variables = liquid::object!({
                            "suggestion_emoji": if info.suggests {"🌟"} else {""},
                            "suggestion_verb": if info.suggests {"now"} else {"no longer"},
                            "status_verb": info.watch_status.as_verb(),
                            "status_emoji": info.watch_status.as_emoji(),
                        });
                        let reply_message = template.render(&variables)?;
                        interaction
                            .reply_success(ctx, reply_message, current_media.get_title())
                            .await?;
                    }
                }
                Some(Err(why)) => {
                    interaction.reply_error(ctx, "There is an error‼").await?;
                    return Err(anyhow!("{:?}", why).into());
                }
                None => interaction.ack(ctx).await?,
            },
            _ => (),
        }
    }

    // remove components after timeout
    anime_card
        .edit(&ctx, |m| m.set_components(CreateComponents::default()))
        .await?;

    Ok(())
}
