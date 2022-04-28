mod media_paginator;
pub use media_paginator::MediaPaginator;

use anyhow::Result;
use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateButton, CreateComponents, CreateEmbed},
    model::{id::EmojiId, interactions::message_component::ButtonStyle},
};

pub trait AsEmbed {
    fn as_embed(&self) -> CreateEmbed;
}

pub trait AsComponent {
    fn as_component(&self) -> CreateComponents;
}

#[async_trait]
pub trait EmbedPaginator {
    type Item: AsEmbed;
    type Page;

    async fn get_page(&self, page: usize) -> Result<Self::Page>;
    fn has_next(&self) -> bool;
    fn has_prev(&self) -> bool;

    fn current_item(&self) -> Option<Self::Item>;
    async fn next_item(&mut self) -> Option<Self::Item>;
    async fn prev_item(&mut self) -> Option<Self::Item>;

    fn build_button_components(&self, _: &mut CreateActionRow) {}
}

impl<T: EmbedPaginator> AsComponent for T {
    fn as_component(&self) -> CreateComponents {
        let mut action_row = CreateActionRow::default();
        action_row.add_button(
            CreateButton::default()
                .style(ButtonStyle::Secondary)
                .custom_id("PREV_PAGE")
                .emoji(EmojiId(877152666046832670))
                .disabled(!self.has_prev())
                .to_owned(),
        );
        self.build_button_components(&mut action_row);
        action_row.add_button(
            CreateButton::default()
                .style(ButtonStyle::Secondary)
                .custom_id("NEXT_PAGE")
                .emoji(EmojiId(877152666080387122))
                .disabled(!self.has_next())
                .to_owned(),
        );

        let mut components = CreateComponents::default();
        components.add_action_row(action_row);
        components
    }
}
