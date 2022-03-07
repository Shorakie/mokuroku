use std::time::Duration;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    collector::reaction_collector::ReactionAction,
    model::channel::{Message, ReactionType,},
    utils::Colour,
    futures::StreamExt,
    prelude::Context,
};
use tracing::{info, error};
use html2md::parse_html;

use crate::services::anime::{AnimeQueryError::AnimeNotFoundError, find_anime, get_animes};
use crate::services::mongo::{
    set_watching_status,
    remove_watching,
    suggest,
    get_watching,
};
use crate::strings::anime::card;
use crate::utils::extract_user_id;

#[command]
pub async fn anime(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // validate arguments
    let args = args.trimmed().quoted();
    if args.len() < 1 {
        msg.channel_id.say(&ctx.http, "Anime command requires an argument\n`watch <anime name>`").await?;
        return Ok(())
    }

    // find anime
    let anime_name = args.rest();
    info!("searching for anime: {}", anime_name);
    let anime = match find_anime(anime_name).await {
        Ok(anime) => anime,
        Err(query_error) => match query_error {
            AnimeNotFoundError(_not_found_err) => {
                msg.channel_id.say(&ctx.http, "Cannot find any anime...").await?;
                return Ok(())
            },
            _ => {
                msg.channel_id.say(&ctx.http, "❗ There was an unexpected error...").await?;
                return Ok(())
            }
        }
    };

    // TODO: implement paging
    // send anime card
    let anime_card = match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&anime.get_title())
                .color(Colour::new(0x345A78))
                .thumbnail(&anime.cover_image.medium)
                .description(parse_html(&anime.description))
                .fields(vec![
                    (card::STATUS, &anime.status.to_string(), true),
                    (card::FORMAT, &anime.format.to_string(), true),
                ])
                .field(card::GENRES, &anime.genres.join(", "), false)
                .field(
                    card::AIRED,
                    &format!("from **{}** to **{}**", &anime.start_date.to_string(), &anime.end_date.to_string()),
                    false
                ).fields(vec![
                    (
                        card::EPISODES,
                        &anime.episodes.map_or("?".to_owned(), |episodes| episodes.to_string()),
                        true
                    ),
                    (
                        card::DURATION,
                        &format!("{} min", &anime.duration.map_or("?".to_owned(), |duration| duration.to_string())),
                        true
                    ),
                    (
                        card::RATING,
                        &format!("**{}/100**", &anime.average_score.map_or("?".to_owned(), |average_score| average_score.to_string())),
                        true
                    ),
                ])
        })
        .reactions(
            vec!["⬅️", "👀", "🏁", "👌", "➡️"]
            .into_iter()
            .map(|emoji| ReactionType::Unicode(emoji.into()))
            .collect::<Vec<ReactionType>>())
    })
    .await {
        Ok(message) => message,
        Err(why) => {
            error!("Error sending message: {:?}", why);
            return Ok(())
        },
    };

    // listen for reactions and take action
    // prev ⬅️ watch 👀 finish 🏁 suggest 👌 next ➡️
    while let Some(reaction_action) = anime_card
        .await_reactions(&ctx)
        .added(true)
        .removed(true)
        .message_id(anime_card.id)
        .channel_id(anime_card.channel_id)
        .filter(|reaction| ["⬅️", "👀", "🏁", "👌", "➡️"].iter().any(|emoji| reaction.emoji.unicode_eq(emoji)))
        .timeout(Duration::from_secs(30))
        .await
        .next()
        .await
    {
        match *reaction_action {
            ReactionAction::Added(ref reaction) => {
                // get he the sender if present
                // else remove reaction and process next
                let user_id = match reaction.user_id {
                    Some(user_id) => user_id,
                    None => {
                        reaction.delete(&ctx).await?;
                        continue
                    },
                };

                if reaction.emoji.unicode_eq("⬅️") {
                    // TODO: implement previous anime
                } else if reaction.emoji.unicode_eq("➡️") {
                    // TODO: implement next anime
                } else if reaction.emoji.unicode_eq("👀") {
                    info!("adding {} to <!@{}>'s watch list", &anime.get_title(), &user_id);
                    set_watching_status(user_id, anime.id.into(), "WATCHING").await?;
                    anime_card.reply(&ctx, format!("<@{}> is watching **{}** 👀", &user_id, &anime.get_title())).await?;
                } else if reaction.emoji.unicode_eq("🏁") {
                    info!("adding {} to <!@{}>'s finished list", &anime.get_title(), &user_id);
                    set_watching_status(user_id, anime.id.into(), "FINISHED").await?;
                    anime_card.reply(&ctx, format!("<@{}> finished **{}** 🏁", &user_id, &anime.get_title())).await?;
                } else if reaction.emoji.unicode_eq("👌") {
                    info!("adding {} to <!@{}>'s suggest list", &anime.get_title(), &user_id);
                    suggest(user_id, anime.id.into()).await?;
                    anime_card.reply(&ctx, format!("<@{}> suggests **{}** 👌", &user_id, &anime.get_title())).await?;
                }
                // auto remove page reactions
                if ["⬅️", "➡️"].iter().any(|emoji| reaction.emoji.unicode_eq(emoji)) {
                    reaction.delete(&ctx).await?;
                }
            },
            ReactionAction::Removed(ref reaction) => {
                // get he the sender if not present default to Bot id
                let user_id = match reaction.user_id {
                    Some(user_id) => user_id,
                    None => anime_card.author.id,
                };

                if reaction.emoji.unicode_eq("👀") {
                    info!("removing {} from <!@{}>'s watch list", &anime.get_title(), &user_id);
                    remove_watching(user_id, anime.id.into()).await?;
                    anime_card.reply(&ctx, format!("<@{}> is not watching **{}** 🙈", &user_id, &anime.get_title())).await?;
                } else if reaction.emoji.unicode_eq("🏁") {
                    info!("removing {} from <!@{}>'s finish list", &anime.get_title(), &user_id);
                    remove_watching(user_id, anime.id.into()).await?;
                    anime_card.reply(&ctx, format!("<@{}> has not finished **{}** 🙈", &user_id, &anime.get_title())).await?;
                } else if reaction.emoji.unicode_eq("👌") {
                    info!("removing {} from <!@{}>'s suggest list", &anime.get_title(), &user_id);
                    suggest(user_id, anime.id.into()).await?;
                    anime_card.reply(&ctx, format!("<@{}> un-suggested **{}** ✋", &user_id, &anime.get_title())).await?;
                }
            },
        }
    }

    Ok(())
}

#[command]
pub async fn watching(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = match args.trimmed().quoted().single::<String>() {
        Ok(user_id) => extract_user_id(&user_id).unwrap_or(msg.author.id),
        Err(_) => msg.author.id,
    };

    let mut watching_anime_list = get_watching(user_id, None).await?;

    // paginate ids
    let paginated_anime_ids: Vec<i64> = if watching_anime_list.len() > 10 {
        watching_anime_list.drain(..10).collect()
    } else {
        watching_anime_list.drain(..).collect()
    };
    
    info!("batching for list of anime: {:?}", &paginated_anime_ids);
    let animes = match get_animes(&paginated_anime_ids).await {
        Some(found_animes) => found_animes,
        None => {
            error!("Cannot find the paginated animes: {:#?}", &paginated_anime_ids);
            return Ok(())
        }
    };

    // check if we have found any
    if let Some(errors) = animes["errors"].as_array() {
        if errors.len() > 0 {
            info!("there were some errors retrieving animes info: {:#?}", errors);
            msg.channel_id.say(&ctx.http, "❗Error retrieving anime watch list...").await?;
            return Ok(())
        }
    }

    // convert the data into array
    let animes = match animes["data"].as_object() {
        Some(data) => data.values().collect::<Vec<&serde_json::Value>>(),
        None => {
            info!("there were no anime in the data field");
            msg.channel_id.say(&ctx.http, "❗Cannot find any anime in your watch list...").await?;
            return Ok(())
        },
    };
    
    msg.channel_id.send_message(&ctx, |m| {m.embed(|e| {
        e.title(format!("<@!{}>'s Watch List", &user_id))
            .color(Colour::new(0x345A78))
            .fields(animes.iter().map(|anime| (anime["title"]["userPreferred"].as_str().unwrap_or("Unknown"), "<@!381411262321393665>", false)).collect::<Vec<(&str, &str, bool)>>())
    })}).await?;

    Ok(())
}

#[command]
pub async fn finished(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {Ok(())}

#[command]
pub async fn suggesting(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {Ok(())}
