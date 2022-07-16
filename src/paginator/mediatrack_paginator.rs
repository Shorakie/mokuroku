use std::cmp::max;

use anyhow::{bail, Result};
use graphql_client::reqwest::post_graphql;
use mongodm::prelude::MongoCollection;
use reqwest::Client;
use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateButton},
    model::{id::UserId, interactions::message_component::ButtonStyle},
};
use tracing::{error, info};

use crate::{
    config,
    db::{
        mediatrack::{MediaTrack, MediaTrackSeason},
        watchlist::{WatchInfo, WatchStatus},
    },
    graphql::{
        lookup_media_page::{
            self, LookupMediaPagePage as MediaPage, LookupMediaPagePageMedia as Media,
            LookupMediaPagePagePageInfo as PageInfo, MediaType,
        },
        LookupMediaPage,
    },
};

use super::EmbedPaginator;

#[derive(Default, Builder)]
pub struct MediaTrackPaginatorOptions {
    media_switch: bool,
    suggestion_toggle: bool,
}

pub struct MediaTrackPaginator {
    items: Vec<Media>,
    index: usize,
    // write only, used for quering graphql
    query_variables: lookup_media_page::Variables,
    // read only, used for state checking
    page_info: PageInfo,
}

impl MediaTrackPaginator {
    pub async fn new(
        collection: MongoCollection<WatchInfo>,
        user_id: UserId,
        watch_status: impl IntoIterator<Item = WatchStatus>,
        media_type: impl IntoIterator<Item = MediaType>,
        suggests: Option<bool>,
    ) -> Result<Self> {
        todo!()
    }
}

#[async_trait]
impl EmbedPaginator for MediaTrackPaginator {
    type Item = MediaTrackSeason;
    type Page = Vec<Self::Item>;

    async fn get_page(&self, page: usize) -> Result<Self::Page> {
        info!("querying page of");

        todo!();
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
                .disabled(
                    !self
                        .current_item()
                        .map(|media| media.has_start_date())
                        .unwrap_or(false),
                )
                .to_owned(),
        );
        action_row.add_button(
            CreateButton::default()
                .style(ButtonStyle::Primary)
                .custom_id("FINISH")
                .emoji('🏁')
                .disabled(
                    !self
                        .current_item()
                        .map(|media| media.has_start_date())
                        .unwrap_or(false),
                )
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
