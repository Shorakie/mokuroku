use serde::Deserialize;
use serenity::model::id::ApplicationId;

#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub discord_token: String,
    pub discord_bot_id: Option<ApplicationId>,

    #[serde(default = "default_prefix")]
    pub command_prefix: String,
}

fn default_prefix() -> String {
    "mr!".to_string()
}
