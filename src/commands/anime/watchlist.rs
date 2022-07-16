use mongodm::{doc, ToRepository};
use serenity::{
    client,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::UserId},
};
use std::fmt::Write;

use crate::{
    db::{
        mediatrack::MediaTrackSeasonExt,
        watchlist::{WatchInfo, WatchStatus},
    },
    embeds::make_blank_embed,
    extentions::ClientContextExt,
    graphql::lookup_media_page::MediaType,
};

#[command]
#[usage("watching [<user>]")]
#[max_args(1)]
pub async fn watching(ctx: &client::Context, msg: &Message, mut args: Args) -> CommandResult {
    // find the user to get the watch list
    let user = match args.single::<UserId>() {
        Ok(id) => id
            .to_user(&ctx)
            .await
            .unwrap_or_else(|_| msg.author.clone()),
        Err(_) => msg.author.clone(),
    };

    let watch_info_repo = ctx.get_db().await.repository::<WatchInfo>();

    // TODO: use configurable value for watch_status and media_type
    let page = watch_info_repo
        .get_media_track_page(doc! {
            "discord_user_id": user.id.0 as i64,
            // "start_date": { LesserThan: Utc::now() },
            "watch_status": WatchStatus::Consuming,
            "media_type": MediaType::Anime,
        })
        .await?;

    msg.channel_id
        .send_message(&ctx, |m| {
            m.set_embed(make_blank_embed(|e| {
                e.title("Watching 👀");
                e.description(format!("list of anime **{}** is watching 👀", user.name));
                e.fields(page.into_iter().map(|season| {
                    let mut list = String::new();
                    for media in season.media {
                        let _ = writeln!(&mut list, "{}", media);
                    }
                    (season.release_season, list, false)
                }))
            }))
        })
        .await?;
    // finished == watch (switch media, toggle suggestions)
    // suggests (switch media)
    Ok(())
}
