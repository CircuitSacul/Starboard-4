use std::fmt::Write;

use lazy_static::lazy_static;
use regex::Regex;
use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        embed::Embed,
        Component,
    },
    id::{marker::MessageMarker, Id},
    util::Timestamp,
};
use twilight_util::{
    builder::embed::{
        EmbedAuthorBuilder, EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder, ImageSource,
    },
    snowflake::Snowflake,
};

use common::constants;
use errors::StarboardResult;

use crate::{
    cache::{models::message::CachedMessage, MessageResult},
    core::emoji::{EmojiCommon, SimpleEmoji},
    utils::{
        avatar::ImageHashAvatar, id_as_i64::GetI64, into_id::IntoId, message_link::fmt_message_link,
    },
};

use super::{parser::ParsedMessage, AttachmentHandle, Embedder};

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(concat!(
        r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]",
        r"{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&/=]*)$"
    ))
    .unwrap();
}

pub struct FullBuiltStarboardEmbed {
    pub top_content: String,
    pub embeds: Vec<Embed>,
    pub upload_attachments: Vec<AttachmentHandle>,
    pub components: Vec<Component>,
}

pub struct PartialBuiltStarboardEmbed {
    pub top_content: String,
}

pub enum BuiltStarboardEmbed {
    Full(FullBuiltStarboardEmbed),
    Partial(PartialBuiltStarboardEmbed),
}

impl BuiltStarboardEmbed {
    pub async fn build(
        handle: &Embedder,
        force_partial: bool,
        watermark: bool,
    ) -> StarboardResult<Self> {
        if let MessageResult::Ok(orig) = &handle.orig_message {
            if !force_partial {
                let parsed = ParsedMessage::parse(orig);

                let built = Self::Full(FullBuiltStarboardEmbed {
                    top_content: Self::build_top_content(handle),
                    embeds: Self::build_embeds(handle, orig, &parsed, watermark).await?,
                    upload_attachments: parsed.upload_attachments,
                    components: Self::build_components(handle),
                });
                return Ok(built);
            }
        }

        let built = Self::Partial(PartialBuiltStarboardEmbed {
            top_content: Self::build_top_content(handle),
        });
        Ok(built)
    }

    pub fn build_go_to_message_button(handle: &Embedder) -> Option<Button> {
        if handle.config.resolved.go_to_message != 2 {
            return None;
        }

        Some(Button {
            custom_id: None,
            disabled: false,
            emoji: None,
            label: Some("Go to Message".to_string()),
            style: ButtonStyle::Link,
            url: Some(fmt_message_link(
                handle.config.starboard.guild_id,
                handle.orig_sql_message.channel_id,
                handle.orig_sql_message.message_id,
            )),
        })
    }

    pub fn build_components(handle: &Embedder) -> Vec<Component> {
        let Some(gtm) = Self::build_go_to_message_button(handle) else {
            return vec![];
        };

        vec![Component::ActionRow(ActionRow {
            components: vec![Component::Button(gtm)],
        })]
    }

    pub fn build_top_content(handle: &Embedder) -> String {
        let mut top_content = String::new();

        if let Some(emoji) = handle.config.resolved.display_emoji.clone() {
            let emoji = SimpleEmoji::from_stored(emoji);
            top_content.push_str(
                &emoji.into_readable(&handle.bot, handle.config.starboard.guild_id.into_id()),
            );
        }
        write!(top_content, " **{} |** ", handle.points,).unwrap();
        if handle.config.resolved.go_to_message == 3 {
            write!(
                top_content,
                "{}",
                fmt_message_link(
                    handle.config.starboard.guild_id,
                    handle.orig_sql_message.channel_id,
                    handle.orig_sql_message.message_id,
                )
            )
            .unwrap();
        } else {
            write!(top_content, "<#{}>", handle.orig_sql_message.channel_id,).unwrap();
        }
        if handle.config.resolved.ping_author {
            write!(
                top_content,
                " **(**<@{}>**)**",
                handle.orig_sql_message.author_id
            )
            .unwrap();
        }

        if handle.orig_sql_message.frozen {
            write!(top_content, " ❄️").unwrap();
        }
        if handle
            .orig_sql_message
            .forced_to
            .contains(&handle.config.starboard.id)
        {
            write!(top_content, " 🔒").unwrap();
        }

        top_content
    }

    pub async fn build_embeds(
        handle: &Embedder,
        orig: &CachedMessage,
        parsed: &ParsedMessage,
        watermark: bool,
    ) -> StarboardResult<Vec<Embed>> {
        let mut embeds = Vec::new();

        if let Some(e) = Self::build_replied_embed(handle).await? {
            embeds.push(e);
        }
        if let Some(e) = Self::build_primary_embed(handle, orig, parsed, watermark, false).await? {
            embeds.push(e);
        }

        if handle.config.resolved.extra_embeds {
            for e in &parsed.embeds {
                if embeds.len() >= 9 {
                    embeds.push(
                        EmbedBuilder::new()
                            .description(concat!(
                                "This message has too many embeds. ",
                                "Go to the original message to see all content."
                            ))
                            .color(constants::EMBED_DARK_BG)
                            .build(),
                    );
                    break;
                }

                embeds.push(e.clone());
                if embeds.len() >= 10 {
                    break;
                }
            }
        }

        Ok(embeds)
    }

    pub async fn build_replied_embed(handle: &Embedder) -> StarboardResult<Option<Embed>> {
        if !handle.config.resolved.replied_to {
            return Ok(None);
        }

        let ref_msg = match &handle.referenced_message {
            None => return Ok(None),
            Some(msg) => msg,
        };
        let reply_parsed = ParsedMessage::parse(ref_msg);
        Self::build_primary_embed(handle, ref_msg, &reply_parsed, false, true).await
    }

    pub async fn build_primary_embed(
        handle: &Embedder,
        orig: &CachedMessage,
        parsed: &ParsedMessage,
        watermark: bool,
        is_reply: bool,
    ) -> StarboardResult<Option<Embed>> {
        let mut zws_fields: Vec<String> = Vec::new();
        let color = if is_reply {
            constants::EMBED_DARK_BG
        } else {
            handle
                .config
                .resolved
                .color
                .map(|c| c as u32)
                .unwrap_or(constants::BOT_COLOR)
        };
        let mut embed = EmbedBuilder::new().color(color);

        let (link, mid_i64) = {
            let mid = match is_reply {
                true => handle
                    .orig_message
                    .as_option()
                    .unwrap()
                    .referenced_message
                    .unwrap()
                    .get_i64(),
                false => handle.orig_sql_message.message_id,
            };

            (
                fmt_message_link(
                    handle.config.starboard.guild_id,
                    handle.orig_sql_message.channel_id,
                    mid,
                ),
                mid,
            )
        };
        let mid: Id<MessageMarker> = mid_i64.into_id();

        // author
        let (name, avatar) = Self::get_author(handle, is_reply).await?;

        let mut author = EmbedAuthorBuilder::new(name).url(&link);
        if let Some(avatar) = avatar {
            author = author.icon_url(ImageSource::url(avatar).unwrap());
        }

        embed = embed.author(author.build());

        // main description
        let mut description = String::new();

        if let Some(name_str) = &parsed.sticker_names_str {
            description.push_str(name_str);
            description.push('\n');
        }

        'out: {
            if !orig.content.is_empty() {
                if URL_REGEX.is_match(&orig.content) {
                    let url = orig.content.split('?').next().unwrap_or(&orig.content);

                    let mut found = false;
                    for item in &parsed.urls.embedded {
                        if item.url == url {
                            found = true;
                            break;
                        }
                    }

                    if found {
                        break 'out;
                    }
                }

                description.push_str(&orig.content);
            }
        }

        let mut has_description;
        if !description.is_empty() {
            embed = embed.description(description);
            has_description = true;
        } else {
            has_description = false;
        }

        // jump link
        if handle.config.resolved.go_to_message == 1 && !is_reply {
            zws_fields.push(format!("[Go to Message]({link})"));
        }

        // attachments list
        let mut urls = Vec::new();
        urls.extend(parsed.urls.uploaded.iter().map(|url| url.to_md()));
        if !handle.config.resolved.extra_embeds || is_reply && parsed.urls.embedded.len() > 1 {
            urls.extend(parsed.urls.embedded.iter().map(|url| url.to_md()));
        }

        if (handle.config.resolved.attachments_list || is_reply) && !urls.is_empty() {
            let mut field = String::new();

            for next in urls {
                if field.len() + next.len() + 10 > 1_024 {
                    field.push_str("...");
                    break;
                }

                field.push_str(&next);
                field.push('\n');
            }

            zws_fields.push(field);
        }

        // primary image
        if let Some(image) = &parsed.primary_image {
            embed = embed.image(image.clone());
        }

        // timestamp
        embed = embed.timestamp(Timestamp::from_micros(mid.timestamp() * 1000).unwrap());

        // add the fields
        for field in zws_fields {
            if !has_description {
                has_description = true;
                embed = embed.description(field);
            } else {
                embed = embed.field(EmbedFieldBuilder::new(constants::ZWS, field).build());
            }
        }

        // watermark footer
        if watermark {
            embed = embed.footer(EmbedFooterBuilder::new("Powered by https://starboard.best"));
        }

        // build
        Ok(Some(embed.build()))
    }

    async fn get_author(
        handle: &Embedder,
        is_reply: bool,
    ) -> StarboardResult<(String, Option<String>)> {
        let (author, author_id) = if is_reply {
            if let Some(reply) = &handle.referenced_message {
                (reply.author.clone(), reply.author_id)
            } else {
                return Ok(("Replying to Deleted User".to_string(), None));
            }
        } else if let Some(orig) = handle.orig_message.as_option() {
            (orig.author.clone(), orig.author_id)
        } else {
            return Ok(("Deleted User".to_string(), None));
        };

        let guild_id = handle.config.starboard.guild_id.into_id();

        let member = if handle.config.resolved.use_server_profile {
            handle
                .bot
                .cache
                .fog_member(&handle.bot, guild_id, author_id)
                .await?
        } else {
            None
        };

        let avatar = member
            .as_ref()
            .and_then(|m| {
                m.server_avatar_hash
                    .map(|av| av.guild_avatar(author_id, guild_id))
            })
            .or(author.avatar.map(|av| av.global_avatar(author_id)));
        let mut name = member
            .and_then(|m| m.nickname.clone())
            .unwrap_or(author.name);

        if is_reply {
            name = format!("Replying to {name}");
        }

        Ok((name, avatar))
    }
}
