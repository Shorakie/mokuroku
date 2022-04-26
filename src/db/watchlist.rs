use std::fmt::Display;

use anyhow::{Context, Result};
use mongodm::{
    doc,
    operator::{Equal, MergeObjects, Not, ReplaceWith, Set, Unset},
    prelude::{
        to_bson, Bson, BsonDateTime, MongoCollection, MongoFindOneAndUpdateOptions,
        MongoReturnDocument,
    },
    CollectionConfig, Index, IndexOption, Indexes, Model,
};
use serde::{Deserialize, Serialize};
use serenity::{async_trait, model::id::UserId};

use crate::graphql::lookup_media_page::{LookupMediaPagePageMedia as Media, MediaType};

pub struct WatchInfoCollConf;

impl CollectionConfig for WatchInfoCollConf {
    fn collection_name() -> &'static str {
        "watch-list"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new("anilist_media_id")
                .with_key("discord_user_id")
                .with_option(IndexOption::Unique),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WatchStatus {
    NotSeen,
    Consuming,
    Finished,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WatchInfo {
    pub anilist_media_id: i64,
    pub discord_user_id: UserId,
    pub media_type: MediaType,
    pub watch_status: WatchStatus,
    pub last_watch_status: WatchStatus,
    pub suggests: bool,
    pub rating: Option<u8>,
    pub updated_at: BsonDateTime,
    pub created_at: BsonDateTime,
}

impl Model for WatchInfo {
    type CollConf = WatchInfoCollConf;
}

impl From<WatchStatus> for Bson {
    fn from(val: WatchStatus) -> Self {
        match val {
            WatchStatus::Finished => Bson::String("FINISHED".to_owned()),
            WatchStatus::Consuming => Bson::String("CONSUMING".to_owned()),
            WatchStatus::NotSeen => Bson::String("NOT_SEEN".to_owned()),
        }
    }
}

impl From<String> for WatchStatus {
    fn from(val: String) -> Self {
        match val.as_str() {
            "FINISHED" => WatchStatus::Finished,
            "CONSUMING" => WatchStatus::Consuming,
            _ => WatchStatus::NotSeen, // includes "NOT_SEEN"
        }
    }
}

impl Display for WatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchStatus::Finished => write!(f, "Finished {}", self.as_emoji()),
            WatchStatus::Consuming => write!(f, "Watching {}", self.as_emoji()),
            WatchStatus::NotSeen => write!(f, "Not seen {}", self.as_emoji()),
        }
    }
}

impl WatchStatus {
    pub fn as_emoji(&self) -> &str {
        match self {
            WatchStatus::Finished => "🏁",
            WatchStatus::Consuming => "👀",
            WatchStatus::NotSeen => "❓",
        }
    }

    pub fn as_verb(&self) -> &str {
        match self {
            WatchStatus::Finished => "finished",
            WatchStatus::Consuming => "are watching",
            WatchStatus::NotSeen => "forgot",
        }
    }
}

#[async_trait]
pub trait WatchListCollectionExt {
    async fn toggle_suggestion(&self, media: &Media, user_id: UserId) -> Result<Option<WatchInfo>>;
    async fn toggle_finish(&self, media: &Media, user_id: UserId) -> Result<Option<WatchInfo>>;
    async fn toggle_consuming(&self, media: &Media, user_id: UserId) -> Result<Option<WatchInfo>>;
}

#[async_trait]
impl WatchListCollectionExt for MongoCollection<WatchInfo> {
    async fn toggle_finish(&self, media: &Media, user_id: UserId) -> Result<Option<WatchInfo>> {
        let options = MongoFindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(MongoReturnDocument::After)
            .build();
        self.find_one_and_update(
            doc! {"anilist_media_id": media.id, "discord_user_id": to_bson(user_id.as_u64()).unwrap()},
            vec![
                doc! { Set: {
                    "temp": "$last_watch_status",
                    "last_watch_status": "$watch_status",
                    "updated_at": "$$NOW" ,
                } },
                doc! { Set: {
                    "watch_status": {
                        "$cond": [ 
                            { Equal: ["$watch_status", WatchStatus::Finished] },
                            "$temp",
                            WatchStatus::Finished
                        ]
                    }
                } },
                doc! { ReplaceWith: { MergeObjects: [
                    {
                        "last_watch_status": WatchStatus::NotSeen,
                        "created_at": "$$NOW",
                        "media_type": MediaType::Anime,
                        "rating": Bson::Null,
                        "suggests": false,
                    },
                    "$$ROOT"
                ] } },
                doc! { Unset: "temp" },
            ],
            Some(options),
        ).await.context("Failed to toggle suggestion")
    }

    async fn toggle_consuming(&self, media: &Media, user_id: UserId) -> Result<Option<WatchInfo>> {
        let options = MongoFindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(MongoReturnDocument::After)
            .build();
        self.find_one_and_update(
            doc! {"anilist_media_id": media.id, "discord_user_id": to_bson(user_id.as_u64()).unwrap()},
            vec![
                doc! { Set: {
                    "temp": "$last_watch_status",
                    "last_watch_status": "$watch_status",
                    "updated_at": "$$NOW" ,
                } },
                doc! { Set: {
                    "watch_status": {
                        "$cond": [ 
                            { Equal: ["$watch_status", WatchStatus::Consuming] },
                            "$temp",
                            WatchStatus::Consuming
                        ]
                    }
                } },
                doc! { ReplaceWith: { MergeObjects:[
                    {
                        "last_watch_status": WatchStatus::NotSeen,
                        "created_at": "$$NOW",
                        "media_type": MediaType::Anime,
                        "rating": Bson::Null,
                        "suggests": false,
                    },
                    "$$ROOT"
                ] } },
                doc! { Unset: "temp" },
            ],
            Some(options),
        ).await.context("Failed to toggle suggestion")
    }

    async fn toggle_suggestion(&self, media: &Media, user_id: UserId) -> Result<Option<WatchInfo>> {
        let options = MongoFindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(MongoReturnDocument::After)
            .build();
        self.find_one_and_update(
            doc! {"anilist_media_id": media.id, "discord_user_id": to_bson(user_id.as_u64()).unwrap()},
            vec![
                doc! { Set: { "suggests": { Not: "$suggests" }, "updated_at": "$$NOW" }, },
                doc! { ReplaceWith: { MergeObjects:[
                    {
                        "created_at": "$$NOW",
                        "media_type": MediaType::Anime,
                        "rating": Bson::Null,
                        "watch_status": WatchStatus::NotSeen,
                        "last_watch_status": WatchStatus::NotSeen,
                    },
                    "$$ROOT"
                ] } },
            ],
            Some(options),
        ).await.context("Failed to toggle suggestion")
    }
}
