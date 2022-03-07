use lazy_static::lazy_static;
use regex::Regex;

use serenity::{
    model::id::UserId,
};

// hello <@!136456456765431>
// TODO: return Result
pub fn extract_user_id(input: &str) -> Option<UserId> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^<@!(?P<user_id>\d+)>$").unwrap();
    }
    let user_id = match RE.captures(input).and_then(|cap| {
        cap.name("user_id").map(|user_id| user_id.as_str())
    }) {
        Some(user_id) => user_id,
        None => return None,
    };

    match user_id.parse::<u64>() {
        Ok(user_id) => Some(user_id.into()),
        Err(_) => None,
    }
}