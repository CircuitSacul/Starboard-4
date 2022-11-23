use twilight_model::channel::message::{embed::Embed, sticker::StickerFormatType};
use twilight_util::builder::embed::ImageSource;

use crate::cache::models::message::CachedMessage;

use super::{image_only_embed::maybe_get_attachment_handle, AttachmentHandle, Embedder};

pub struct ParsedMessage {
    pub sticker_names_str: Option<String>,
    // attachments
    pub url_list: Vec<String>,
    pub primary_image: Option<ImageSource>,
    pub embeds: Vec<Embed>,
    pub upload_attachments: Vec<AttachmentHandle>,
}

impl ParsedMessage {
    pub fn parse(_handle: &Embedder, orig: &CachedMessage) -> Self {
        let (sticker_names_str, primary_image, url_list, embeds, upload_attachments) =
            Self::parse_attachments(orig);

        Self {
            sticker_names_str,
            primary_image,
            url_list,
            embeds,
            upload_attachments,
        }
    }

    pub fn parse_attachments(
        orig: &CachedMessage,
    ) -> (
        Option<String>,
        Option<ImageSource>,
        Vec<String>,
        Vec<Embed>,
        Vec<AttachmentHandle>,
    ) {
        let mut primary_image = None;
        let mut embeds = Vec::new();
        let mut upload_attachments = Vec::new();
        let mut url_list = Vec::new();

        for attachment in &orig.attachments {
            let handle = AttachmentHandle::from_attachment(attachment);
            url_list.push(handle.url_list_item());

            if primary_image.is_none() {
                if let Some(image) = handle.embedable_image() {
                    primary_image.replace(image);
                    continue;
                }
            } else if let Some(embed) = handle.as_embed() {
                embeds.push(embed);
                continue;
            }

            upload_attachments.push(handle);
        }

        for embed in &orig.embeds {
            if let Some(attachment) = maybe_get_attachment_handle(embed) {
                if let Some(image) = attachment.embedable_image() {
                    if primary_image.is_none() && embeds.is_empty() {
                        primary_image.replace(image);
                    } else {
                        embeds.push(attachment.as_embed().unwrap());
                    }
                } else {
                    upload_attachments.push(attachment);
                }
            } else {
                embeds.push(embed.clone());
            }
        }

        let sticker_names_str: Option<String>;
        if !orig.stickers.is_empty() {
            let mut sticker_names = Vec::new();

            for sticker in &orig.stickers {
                match sticker.format_type {
                    StickerFormatType::Lottie => {
                        sticker_names.push(format!("Sticker: **{}**", sticker.name));
                    }
                    StickerFormatType::Apng | StickerFormatType::Png => {
                        let handle = AttachmentHandle {
                            filename: format!("{}.png", sticker.name),
                            content_type: Some("image/png".to_string()),
                            url: format!("https://cdn.discordapp.com/stickers/{}.png", sticker.id),
                        };

                        if primary_image.is_none() {
                            if let Some(image) = handle.embedable_image() {
                                primary_image.replace(image);
                                continue;
                            }
                        }

                        if let Some(embed) = handle.as_embed() {
                            embeds.push(embed);
                            continue;
                        }

                        upload_attachments.push(handle);
                    }
                    _ => {}
                }
            }

            if sticker_names.is_empty() {
                sticker_names_str = None;
            } else {
                sticker_names_str = Some(sticker_names.join("\n"))
            }
        } else {
            sticker_names_str = None;
        }

        (
            sticker_names_str,
            primary_image,
            url_list,
            embeds,
            upload_attachments,
        )
    }
}
