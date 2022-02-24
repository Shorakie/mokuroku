use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

const HELP_MESSAGE: &str = "
Hello there, Human!
You have summoned me. Let's see about getting you what you need.
❓ Need technical help?
➡️ Post in the <#946196407285604382> channel and other humans will assist you.
❓ Looking for the Code of Conduct?
➡️ Here it is: <https://opensource.facebook.com/code-of-conduct>
❓ Something wrong?
➡️ You can flag an admin with @admin
I hope that resolves your issue!
— HelpBot 🤖
";

#[command]
pub async fn help(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, HELP_MESSAGE).await?;

    Ok(())
}