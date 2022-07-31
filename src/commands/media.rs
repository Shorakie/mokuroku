use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::info;

#[command("anime")]
#[usage("anime <anime name>")]
#[min_args(1)]
pub async fn anime(_ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    // validate arguments
    let anime_name = args.trimmed().quoted().rest();
    info!("{:?}", anime_name);

    // query media
    // build media card
    // show media card
    // listen for reactions

    Ok(())
}
