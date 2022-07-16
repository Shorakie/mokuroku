use std::fmt::Display;

use anyhow::{Context, Result};
use mongodm::{
    doc,
    operator::{
        AddFields, Divide, Equal, Group, IfNull, Limit, Match, Modulo, Month, Project, Push, Sort,
        ToInt, ToString, Year,
    },
    prelude::{from_bson, Bson, BsonDocument, MongoCollection},
};
use serde::{Deserialize, Serialize};
use serenity::{async_trait, futures::TryStreamExt};

use crate::{db::watchlist::WatchInfo, graphql::lookup_media_page::MediaType};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "season", content = "year")]
pub enum Season {
    #[serde(rename = "0")]
    Spring(i32),
    #[serde(rename = "1")]
    Summer(i32),
    #[serde(rename = "2")]
    Fall(i32),
    #[serde(rename = "3")]
    Winter(i32),
    Upcomming,
}

impl Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spring(year) => write!(f, "🌸 Spring {}", year),
            Self::Summer(year) => write!(f, "🌞 Summer {}", year),
            Self::Fall(year) => write!(f, "🍂 Fall {}", year),
            Self::Winter(year) => write!(f, "❄ Winter {}", year),
            Self::Upcomming => write!(f, "📅 Upcomming..."),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MediaTrack {
    pub title: String,
    pub suggests: bool,
    pub media_type: MediaType,
}

impl Display for MediaTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            if self.suggests { "🔸" } else { "▪" },
            self.title
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct MediaTrackSeason {
    pub release_season: Season,
    pub media: Vec<MediaTrack>,
}

#[async_trait]
pub trait MediaTrackSeasonExt {
    async fn get_media_track_page(&self, query: BsonDocument) -> Result<Vec<MediaTrackSeason>>;
}

#[async_trait]
impl MediaTrackSeasonExt for MongoCollection<WatchInfo> {
    async fn get_media_track_page(&self, query: BsonDocument) -> Result<Vec<MediaTrackSeason>> {
        let cursor = self
        .aggregate(
            vec![
                doc! { Match: query },
                doc! { Limit: 16},
                doc! { Group: {
                    "_id": {
                        "year": { Year: "$start_date" },
                        "season": { IfNull: [{
                            ToString: { ToInt: {
                                Divide: [{Modulo: [{"$subtract": [{Month: "$start_date"}, 2]}, 12]}, 3]
                            }}},
                            "Upcomming"
                        ]}
                    },
                    "media": { Push: {
                        "title": "$title",
                        "suggests": "$suggests",
                        "media_type": "$media_type",
                    }}
                }},
                doc! { AddFields: { "upcomming": {Equal: ["$_id.season", "Upcomming"]} }},
                doc! { Sort: { "upcomming": -1, "_id": -1 }},
                doc! { Project: {
                    "_id": false,
                    "media": true,
                    "release_season": "$_id",
                }},
            ],
            None,
        )
        .await
        .context("cannot aggregate from mongonow")?;

        // let media_track_page: Vec<MediaTrackSeason>
        Ok(cursor
            .try_collect::<Vec<BsonDocument>>()
            .await?
            .into_iter()
            .filter_map(|doc| from_bson(Bson::Document(doc)).ok())
            .collect())
    }
}
