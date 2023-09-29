use std::sync::Arc;

use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle, SelectMenu, SelectMenuOption},
        Component, Embed, ReactionType,
    },
    id::{
        marker::{ChannelMarker, MessageMarker, UserMarker},
        Id,
    },
};

use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    interactions::context::{CommandCtx, ComponentCtx},
    utils::div_ceil::div_ceil,
};

use super::wait_for::wait_for_component;

const ITEMS_PER_CHUNK: usize = 25;

enum AnyContext {
    Initial(CommandCtx),
    Select(CommandCtx, ComponentCtx),
}

impl AnyContext {
    fn into_initial_ctx(self) -> CommandCtx {
        match self {
            Self::Initial(ctx) => ctx,
            Self::Select(ctx, _) => ctx,
        }
    }
}

pub struct SelectPaginatorPage {
    pub label: String,
    pub description: Option<String>,
    pub emoji: Option<ReactionType>,
    pub content: Option<String>,
    pub embeds: Option<Vec<Embed>>,
}

pub struct SelectPaginator {
    ctx: AnyContext,
    bot: Arc<StarboardBot>,
    pub pages: Vec<SelectPaginatorPage>,
    pub current: usize,
    pub user: Id<UserMarker>,
    pub channel: Id<ChannelMarker>,
    pub message: Option<Id<MessageMarker>>,
    done: bool,
}

impl SelectPaginator {
    pub async fn run(mut self) -> StarboardResult<()> {
        if self.pages.is_empty() {
            self.done = true;
        }

        loop {
            let message = self.update_page().await?;
            self.message = Some(message);

            if self.done {
                break;
            }

            let select_ctx = wait_for_component(
                self.bot.clone(),
                &[
                    "select_paginator::select",
                    "select_paginator::next",
                    "select_paginator::back",
                ],
                message,
                self.user,
                60 * 6,
            )
            .await;

            let ctx = match select_ctx {
                None => {
                    self.done = true;
                    self.ctx = AnyContext::Initial(self.ctx.into_initial_ctx());
                    continue;
                }
                Some(ctx) => ctx,
            };

            match &*ctx.data.custom_id {
                "select_paginator::select" => {
                    let choice: usize = ctx.data.values[0].parse().unwrap();
                    self.current = choice;
                }
                "select_paginator::next" => {
                    let current = self.get_chunks().1;
                    self.current = self.get_chunk_range(current + 1).0;
                }
                "select_paginator::back" => {
                    let current = self.get_chunks().1;
                    self.current = self.get_chunk_range(current - 1).0;
                }
                _ => unreachable!(),
            }

            self.ctx = AnyContext::Select(self.ctx.into_initial_ctx(), ctx);
        }

        Ok(())
    }

    pub async fn update_page(&mut self) -> StarboardResult<Id<MessageMarker>> {
        let page = &self.pages[self.current];
        let components = self.components();

        let message = match &mut self.ctx {
            AnyContext::Initial(ctx) => {
                if self.message.is_none() {
                    let mut data = ctx.build_resp().components(components);

                    if let Some(content) = &page.content {
                        data = data.content(content);
                    }
                    if let Some(embeds) = &page.embeds {
                        data = data.embeds(embeds.to_owned());
                    }

                    ctx.respond(data.build()).await?.model().await?.id
                } else {
                    let i = ctx.bot.interaction_client().await;
                    let mut update = i
                        .update_response(&ctx.interaction.token)
                        .components(Some(&components))?;

                    if let Some(content) = &page.content {
                        update = update.content(Some(content))?;
                    }
                    if let Some(embeds) = &page.embeds {
                        update = update.embeds(Some(embeds))?;
                    }

                    update.await?.model().await?.id
                }
            }
            AnyContext::Select(_, ctx) => {
                let mut data = ctx.build_resp().components(components);

                if let Some(content) = &page.content {
                    data = data.content(content);
                }
                if let Some(embeds) = &page.embeds {
                    data = data.embeds(embeds.to_owned());
                }
                let data = data.build();

                ctx.edit(data).await?.model().await?.id
            }
        };

        Ok(message)
    }

    /// (last_chunk, current_chunk)
    fn get_chunks(&self) -> (usize, usize) {
        let last_chunk = div_ceil(self.pages.len(), ITEMS_PER_CHUNK) - 1;
        let current_chunk = self.current / ITEMS_PER_CHUNK;

        (last_chunk, current_chunk)
    }

    fn get_chunk_range(&self, chunk: usize) -> (usize, usize) {
        let start = chunk * ITEMS_PER_CHUNK;
        (start, start + ITEMS_PER_CHUNK)
    }

    fn get_current_chunk_range(&self) -> (usize, usize) {
        self.get_chunk_range(self.get_chunks().1)
    }

    fn pagination_buttons(&self) -> Option<Vec<Component>> {
        let (last_chunk, current) = self.get_chunks();

        if last_chunk == 0 {
            return None;
        }

        let buttons = vec![
            Component::Button(Button {
                custom_id: Some("select_paginator::back".to_string()),
                disabled: current == 0 || self.done,
                emoji: None,
                label: Some("<".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some("select_paginator::meta".to_string()),
                disabled: true,
                emoji: None,
                label: Some(format!("{}/{}", current + 1, last_chunk + 1)),
                style: ButtonStyle::Secondary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some("select_paginator::next".to_string()),
                disabled: current == last_chunk || self.done,
                emoji: None,
                label: Some(">".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            }),
        ];

        Some(buttons)
    }

    fn select_component(&self) -> Component {
        let mut options = Vec::new();
        let (start, end) = self.get_current_chunk_range();
        for (idx, page) in self.pages.iter().enumerate().skip(start).take(end - start) {
            let option = SelectMenuOption {
                default: self.current == idx,
                description: page.description.clone(),
                emoji: page.emoji.clone(),
                label: page.label.clone(),
                value: idx.to_string(),
            };
            options.push(option);
        }

        Component::SelectMenu(SelectMenu {
            custom_id: "select_paginator::select".to_string(),
            disabled: self.done || options.len() == 1,
            max_values: Some(1),
            min_values: Some(1),
            options,
            placeholder: None,
        })
    }

    fn components(&self) -> Vec<Component> {
        let mut rows = Vec::new();

        if let Some(buttons) = self.pagination_buttons() {
            rows.push(Component::ActionRow(ActionRow {
                components: buttons,
            }));
        }

        rows.push(Component::ActionRow(ActionRow {
            components: vec![self.select_component()],
        }));

        rows
    }
}

pub struct SelectPaginatorPageBuilder(SelectPaginatorPage);

impl From<SelectPaginatorPageBuilder> for SelectPaginatorPage {
    fn from(val: SelectPaginatorPageBuilder) -> Self {
        val.0
    }
}

impl SelectPaginatorPageBuilder {
    pub fn new(label: String) -> Self {
        Self(SelectPaginatorPage {
            label,
            description: None,
            emoji: None,
            content: None,
            embeds: None,
        })
    }

    pub fn content(mut self, content: String) -> Self {
        self.0.content = Some(content);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.0.description = Some(description);
        self
    }

    pub fn emoji(mut self, emoji: ReactionType) -> Self {
        self.0.emoji = Some(emoji);
        self
    }

    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.0.embeds = Some(embeds);
        self
    }

    pub fn add_embed(mut self, embed: impl Into<Embed>) -> Self {
        let embeds = match &mut self.0.embeds {
            None => {
                self.0.embeds = Some(Vec::new());
                self.0.embeds.as_mut().unwrap()
            }
            Some(embeds) => embeds,
        };

        embeds.push(embed.into());
        self
    }
}

pub struct SelectPaginatorBuilder(SelectPaginator);

impl SelectPaginatorBuilder {
    pub fn new(ctx: CommandCtx) -> Self {
        Self(SelectPaginator {
            bot: ctx.bot.clone(),
            user: ctx.interaction.author_id().unwrap(),
            channel: ctx.interaction.channel.as_ref().unwrap().id,
            ctx: AnyContext::Initial(ctx),
            pages: Vec::new(),
            current: 0,
            message: None,
            done: false,
        })
    }

    pub fn build(self) -> SelectPaginator {
        self.0
    }

    pub fn add_page(mut self, page: impl Into<SelectPaginatorPage>) -> Self {
        self.0.pages.push(page.into());
        self
    }

    pub fn message(mut self, message: Id<MessageMarker>) -> Self {
        self.0.message = Some(message);
        self
    }

    pub fn current(mut self, current: usize) -> Self {
        self.0.current = current;
        self
    }
}
