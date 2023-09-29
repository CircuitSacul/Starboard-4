use twilight_model::channel::{message::embed::Embed, Attachment};

pub fn has_image(embeds: &Vec<Embed>, attachments: &Vec<Attachment>) -> bool {
    for attachment in attachments {
        if let Some(content_type) = &attachment.content_type {
            if content_type.starts_with("image") {
                return true;
            }
        }
    }

    for embed in embeds {
        if embed.image.is_some() || embed.thumbnail.is_some() {
            return true;
        }
    }

    false
}
