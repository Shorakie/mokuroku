use std::fmt;

use graphql_client::GraphQLQuery;
use html2md::parse_html;
use mongodm::prelude::Bson;
use serenity::{builder::CreateEmbed, utils::Colour};

use crate::{paginator::AsEmbed, strings::card};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/anilist/schema.graphql",
    query_path = "graphql/anilist/media/lookup_media_page.graphql",
    normalization = "Rust",
    variables_derives = "Clone",
    response_derives = "Debug,Clone,PartialEq,Eq"
)]
pub struct LookupMediaPage;

impl Default for lookup_media_page::LookupMediaPagePagePageInfo {
    fn default() -> Self {
        Self {
            total: 0,
            current_page: 1,
            last_page: 1,
            has_next_page: false,
        }
    }
}

impl Default for lookup_media_page::MediaStatus {
    fn default() -> Self {
        Self::Other("Unknown".to_owned())
    }
}

impl Default for lookup_media_page::MediaFormat {
    fn default() -> Self {
        Self::Tv
    }
}

impl fmt::Display for lookup_media_page::MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotYetReleased => write!(f, "Not Released Yet"),
            Self::Hiatus => write!(f, "Hiatus"),
            Self::Finished => write!(f, "Finished"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Releasing => write!(f, "Releasing"),
            _ => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for lookup_media_page::MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tv => write!(f, "📺 Tv"),
            Self::TvShort => write!(f, "📺 Tv Short"),
            Self::Movie => write!(f, "🎥 Movie"),
            Self::Special => write!(f, "🌟 Special"),
            Self::Ova => write!(f, "💽 OVA"),
            Self::Ona => write!(f, "💻 ONA"),
            Self::Music => write!(f, "🎵 Music"),
            Self::Manga => write!(f, "📔 Manga"),
            Self::Novel => write!(f, "📙 Novel"),
            Self::OneShot => write!(f, "📄 One Shot"),
            _ => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for lookup_media_page::LookupMediaPagePageMediaStartDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.year.and(self.month).and(self.day) {
            None => write!(f, "?"),
            _ => write!(
                f,
                "{}-{}-{}",
                self.day.unwrap(),
                self.month.unwrap(),
                self.year.unwrap()
            ),
        }
    }
}

impl fmt::Display for lookup_media_page::LookupMediaPagePageMediaEndDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.year.and(self.month).and(self.day) {
            None => write!(f, "?"),
            _ => write!(
                f,
                "{}-{}-{}",
                self.day.unwrap(),
                self.month.unwrap(),
                self.year.unwrap()
            ),
        }
    }
}

impl From<lookup_media_page::MediaType> for Bson {
    fn from(val: lookup_media_page::MediaType) -> Self {
        match val {
            lookup_media_page::MediaType::Anime => Bson::String("ANIME".to_owned()),
            lookup_media_page::MediaType::Manga => Bson::String("MANGA".to_owned()),
            lookup_media_page::MediaType::Other(_) => Bson::Null,
        }
    }
}

impl lookup_media_page::LookupMediaPagePageMedia {
    pub fn get_title(&self) -> String {
        let title = self.title.clone().unwrap();
        title
            .user_preferred
            .or(title.english)
            .or(title.romaji)
            .or(title.native)
            .unwrap_or_else(|| "?".to_owned())
    }
}

impl AsEmbed for lookup_media_page::LookupMediaPagePageMedia {
    fn as_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(self.get_title());
        embed.color(Colour::new(0x345A78));
        if let Some(thumbnail) = &self.cover_image.as_ref().unwrap().medium {
            embed.thumbnail(thumbnail);
        }
        embed.description(parse_html(
            self.description.as_deref().unwrap_or("No description"),
        ));
        embed.field(card::STATUS, self.status.clone().unwrap_or_default(), true);
        embed.field(card::FORMAT, self.format.clone().unwrap_or_default(), true);
        embed.fields(vec![
            (
                card::GENRES,
                self.genres
                    .clone()
                    .unwrap_or_else(|| vec![Some("?".to_owned())])
                    .into_iter()
                    .flatten()
                    .collect::<Vec<String>>()
                    .join(", "),
                false,
            ),
            (
                card::AIRED,
                format!(
                    "**{}** to **{}**",
                    &self.start_date.as_ref().unwrap(),
                    &self.end_date.as_ref().unwrap()
                ),
                false,
            ),
        ]);
        embed.fields(vec![
            (
                card::EPISODES,
                self.episodes
                    .map_or_else(|| "?".to_owned(), |episodes| episodes.to_string()),
                true,
            ),
            (
                card::DURATION,
                format!(
                    "{} min",
                    self.duration
                        .map_or_else(|| "?".to_owned(), |duration| duration.to_string())
                ),
                true,
            ),
            (
                card::RATING,
                format!(
                    "**{}/100**",
                    self.average_score
                        .map_or_else(|| "?".to_owned(), |rating| rating.to_string())
                ),
                true,
            ),
        ]);

        embed
    }
}
