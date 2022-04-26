use std::cmp::max;

use crate::graphql::{
    lookup_media_page::{self, MediaType},
    LookupMediaPage,
};
use crate::strings::url::ANILIST_API;

use graphql_client::reqwest::post_graphql;
use reqwest::Client;
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::{
    builder::{CreateActionRow, CreateButton},
    model::id::EmojiId,
};
use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum MediaPaginatorError {
    #[error("Request error")]
    RequestError(#[from] reqwest::Error),
    #[error("Error with GraphQL")]
    GraphQLError,
}

type Media = lookup_media_page::LookupMediaPagePageMedia;
type PageInfo = lookup_media_page::LookupMediaPagePagePageInfo;

pub struct MediaPaginator {
    variables: lookup_media_page::Variables,
    page_info: PageInfo,
    media: Vec<Media>,
    index: usize,
}

impl MediaPaginator {
    pub async fn new(
        variables: lookup_media_page::Variables,
    ) -> Result<MediaPaginator, MediaPaginatorError> {
        let mut media_paginator = MediaPaginator {
            variables,
            page_info: PageInfo::default(),
            media: vec![],
            index: 0,
        };
        media_paginator.query().await?;
        Ok(media_paginator)
    }

    async fn query(&mut self) -> Result<(), MediaPaginatorError> {
        info!(
            "querying page {} of {} {}",
            &self.variables.page,
            &self.variables.search,
            match self.variables.media_type {
                MediaType::Anime => "Anime",
                MediaType::Manga => "Manga",
                _ => "unknown type",
            }
        );

        let client = Client::new();

        // Query from ANILIST_API
        let response =
            post_graphql::<LookupMediaPage, _>(&client, ANILIST_API, self.variables.clone())
                .await?;

        // Check for errors
        if let Some(errors) = response.errors {
            error!("GraphQL errors: {:?}", errors);
            return Err(MediaPaginatorError::GraphQLError);
        }

        // get the page
        let page = response.data.unwrap().page.unwrap();
        self.page_info = page.page_info.unwrap();
        self.media = page.media.unwrap().into_iter().flatten().collect();
        Ok(())
    }

    pub fn current_page(&self) -> Option<Media> {
        self.media.get(self.index).cloned()
    }

    pub async fn next_page(&mut self) -> Option<Media> {
        if !self.has_next() {
            return None;
        }

        self.index += 1;
        // if the index is in the next page, query the page first
        if self.index >= self.media.len() {
            self.index = 0;

            // update page number
            self.variables.page += 1;

            self.query().await.ok()?;
        }
        self.media.get(self.index).cloned()
    }

    pub async fn prev_page(&mut self) -> Option<Media> {
        if !self.has_prev() {
            return None;
        }

        if self.index == 0 {
            // update page number
            self.variables.page -= 1;

            self.query().await.ok()?;

            self.index = max(self.media.len() - 1, 0);
        } else {
            self.index -= 1;
        }

        self.media.get(self.index).cloned()
    }

    fn has_next(&self) -> bool {
        self.index + 1 < self.media.len() || self.page_info.has_next_page
    }

    fn has_prev(&self) -> bool {
        self.index > 0 || self.page_info.current_page != 1
    }

    pub fn action_row(&self) -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_button(
            CreateButton::default()
                .style(ButtonStyle::Secondary)
                .custom_id("PREV_PAGE")
                .emoji(EmojiId(877152666046832670))
                .disabled(!self.has_prev())
                .to_owned(),
        );
        ar.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("WATCH")
                .emoji('👀')
                .to_owned(),
        );
        ar.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("FINISH")
                .emoji('🏁')
                .to_owned(),
        );
        ar.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("SUGGEST")
                .emoji('🌟')
                .to_owned(),
        );
        ar.add_button(
            CreateButton::default()
                .style(ButtonStyle::Secondary)
                .custom_id("NEXT_PAGE")
                .emoji(EmojiId(877152666080387122))
                .disabled(!self.has_next())
                .to_owned(),
        );
        ar
    }
}
