use async_trait::async_trait;
use lazy_static::lazy_static;
use twilight_model::{
    channel::{message::embed::Embed, Attachment as ReceivedAttachment},
    http::attachment::Attachment,
};
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

use common::constants;

use errors::{StarboardError, StarboardResult};

use crate::client::bot::StarboardBot;

use super::parser::AttachmentListItem;

pub struct AttachmentHandle {
    pub filename: String,
    pub content_type: Option<String>,
    pub url: String,
}

impl AttachmentHandle {
    pub async fn as_attachment(
        &self,
        bot: &StarboardBot,
        id: u64,
    ) -> StarboardResult<Option<Attachment>> {
        // this should always be a proxy url, but we do this to make 100%
        // sure that there isn't a bug that could potentially leak the VPS ip.
        {
            lazy_static! {
                static ref RE: regex::Regex = regex::Regex::new(
                    r"^https://[\w\.\-]*\.(discord\.com|discordapp\.com|discordapp.net)"
                )
                .unwrap();
            }

            if !RE.is_match(&self.url) {
                return Ok(None);
            }
        }

        // we only want to download files under 8mb
        let head = bot.reqwest.head(&self.url).send().await?;
        let bytes = &head.headers()["content-length"];
        let bytes = bytes.to_str().unwrap().parse::<i64>().unwrap();

        if bytes > 25_000_000 {
            return Ok(None);
        }

        // download the file
        let file = bot.reqwest.get(&self.url).send().await?.bytes().await?;

        Ok(Some(Attachment::from_bytes(
            self.filename.clone(),
            file.to_vec(),
            id,
        )))
    }

    pub fn from_attachment(attachment: &ReceivedAttachment) -> Self {
        let content_type = match attachment.content_type.clone() {
            Some(ct) => Some(ct),
            None => {
                let suffix = attachment.filename.split('.').last();
                suffix.and_then(|suffix| match suffix {
                    "png" | "jpg" | "jpeg" | "gif" | "gifv" => Some(format!("image/{suffix}")),
                    _ => None,
                })
            }
        };

        Self {
            filename: attachment.filename.clone(),
            content_type,
            url: attachment.url.clone(),
        }
    }

    pub fn as_embed(&self) -> Option<Embed> {
        self.embedable_image().map(|image| {
            EmbedBuilder::new()
                .image(image)
                .color(constants::EMBED_DARK_BG)
                .build()
        })
    }

    pub fn attachment_list_item(&self) -> AttachmentListItem {
        if self.filename.len() > 32 {
            AttachmentListItem::new(self.filename[..29].to_string(), self.url.clone())
        } else {
            AttachmentListItem::new(self.filename.clone(), self.url.clone())
        }
    }

    pub fn embedable_image(&self) -> Option<ImageSource> {
        if self.filename.starts_with("SPOILER_") {
            return None;
        }

        if let Some(ct) = &self.content_type {
            if ct.starts_with("image") {
                return Some(ImageSource::url(&self.url).unwrap());
            }
        }

        None
    }
}

#[async_trait]
pub trait VecAttachments {
    async fn as_attachments(&self, bot: &StarboardBot) -> (Vec<Attachment>, Vec<StarboardError>);
}

#[async_trait]
impl VecAttachments for Vec<AttachmentHandle> {
    async fn as_attachments(&self, bot: &StarboardBot) -> (Vec<Attachment>, Vec<StarboardError>) {
        let mut attachments = Vec::new();
        let mut errors = Vec::new();
        for (current_id, attachment) in self.iter().enumerate() {
            match attachment.as_attachment(bot, current_id as u64).await {
                Err(why) => errors.push(why),
                Ok(Some(file)) => attachments.push(file),
                Ok(None) => {}
            }
        }
        (attachments, errors)
    }
}
