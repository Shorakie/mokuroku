use std::cmp::max;

use anyhow::{bail, Result};
use graphql_client::reqwest::post_graphql;
use reqwest::Client;
use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateButton},
    model::interactions::message_component::ButtonStyle,
};
use tracing::{error, info};

use crate::{
    config,
    graphql::{
        lookup_media_page::{
            self, LookupMediaPagePage as MediaPage, LookupMediaPagePageMedia as Media,
            LookupMediaPagePagePageInfo as PageInfo, MediaType,
        },
        LookupMediaPage,
    },
};

use super::EmbedPaginator;

pub struct MediaPaginator {
    items: Vec<Media>,
    index: usize,
    // write only, used for quering graphql
    query_variables: lookup_media_page::Variables,
    // read only, used for state checking
    page_info: PageInfo,
}

impl MediaPaginator {
    pub async fn new(search: String, media_type: MediaType) -> Result<Self> {
        let mut paginator = Self {
            items: vec![],
            index: 0,
            page_info: PageInfo::default(),
            query_variables: lookup_media_page::Variables {
                search,
                page: 1,
                media_type,
                per_page: Some(5),
            },
        };
        let page = paginator
            .get_page(paginator.query_variables.page as usize)
            .await?;
        paginator.page_info = page.page_info.unwrap_or_default();
        paginator.items = page.media.unwrap().into_iter().flatten().collect();
        Ok(paginator)
    }
}

#[async_trait]
impl EmbedPaginator for MediaPaginator {
    type Item = Media;
    type Page = MediaPage;

    async fn get_page(&self, page: usize) -> Result<Self::Page> {
        info!(
            "querying page {} of {} {}",
            &page,
            &self.query_variables.search,
            match self.query_variables.media_type {
                MediaType::Anime => "Anime",
                MediaType::Manga => "Manga",
                _ => "unknown type",
            }
        );

        let client = Client::new();

        // Query from ANILIST_API
        let response = post_graphql::<LookupMediaPage, _>(
            &client,
            config::ANILIST_API,
            // self.query_variables.clone()
            lookup_media_page::Variables {
                page: page as i64,
                ..self.query_variables.clone()
            },
        )
        .await?;

        // Check for errors
        if let Some(errors) = response.errors {
            error!("GraphQL errors: {:?}", errors);
            bail!("GraphQL Error");
        }

        // get the page
        Ok(response.data.unwrap().page.unwrap())
    }

    fn has_next(&self) -> bool {
        self.page_info.has_next_page || self.index + 1 < self.items.len()
    }

    fn has_prev(&self) -> bool {
        !(self.index == 0 && self.page_info.current_page == 1)
    }

    fn current_item(&self) -> Option<Self::Item> {
        self.items.get(self.index).cloned()
    }

    async fn next_item(&mut self) -> Option<Self::Item> {
        if !self.has_next() {
            return None;
        }

        self.index += 1;
        if self.index >= self.items.len() {
            if !self.page_info.has_next_page {
                return None;
            }
            self.query_variables.page += 1;

            let page = self
                .get_page(self.query_variables.page as usize)
                .await
                .ok()?;
            self.page_info = page.page_info.unwrap_or_default();
            self.items = page.media.unwrap().into_iter().flatten().collect();
            self.index = 0;
        }

        self.items.get(self.index).cloned()
    }

    async fn prev_item(&mut self) -> Option<Self::Item> {
        if !self.has_prev() {
            return None;
        }

        if self.index == 0 {
            // update page number
            self.query_variables.page -= 1;

            let page = self
                .get_page(self.query_variables.page as usize)
                .await
                .ok()?;
            self.page_info = page.page_info.unwrap_or_default();
            self.items = page.media.unwrap().into_iter().flatten().collect();
            self.index = max(self.items.len() - 1, 0);
        } else {
            self.index -= 1;
        }

        self.items.get(self.index).cloned()
    }

    fn build_button_components(&self, action_row: &mut CreateActionRow) {
        action_row.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("WATCH")
                .emoji('👀')
                .to_owned(),
        );
        action_row.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("FINISH")
                .emoji('🏁')
                .to_owned(),
        );
        action_row.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("SUGGEST")
                .emoji('🌟')
                .to_owned(),
        );
    }
}
