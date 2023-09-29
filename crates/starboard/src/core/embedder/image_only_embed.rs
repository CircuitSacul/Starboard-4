use twilight_model::channel::message::embed::Embed;

use super::{gifv::get_gif_url, AttachmentHandle};

pub fn maybe_get_attachment_handle(embed: &Embed) -> Option<AttachmentHandle> {
    // gifs
    if &embed.kind == "gifv" {
        let url = get_gif_url(
            &embed.thumbnail.as_ref().unwrap().url,
            embed.provider.as_ref().unwrap().name.as_ref().unwrap(),
        );

        if let Some(url) = url {
            let attachment = AttachmentHandle {
                filename: "GIF".to_string(),
                content_type: Some("image/gif".to_string()),
                url,
            };

            return Some(attachment);
        }
    }

    // catch-all logic
    if !(embed.author.is_none()
        && embed.title.is_none()
        && embed.description.is_none()
        && embed.fields.is_empty()
        && (embed.image.is_some() ^ embed.thumbnail.is_some() || embed.video.is_some()))
    {
        return None;
    }

    let video_url = embed
        .video
        .as_ref()
        .and_then(|v| v.proxy_url.as_ref().or(v.url.as_ref()).cloned());

    let (url, ct) = if let Some(url) = video_url {
        (url, format!("video/{}", embed.kind))
    } else if let Some(image) = &embed.image {
        (
            image.proxy_url.as_ref().unwrap_or(&image.url).clone(),
            format!("image/{}", embed.kind),
        )
    } else if let Some(image) = &embed.thumbnail {
        (
            image.proxy_url.as_ref().unwrap_or(&image.url).clone(),
            format!("image/{}", embed.kind),
        )
    } else {
        unreachable!()
    };

    let name = {
        let name = url.split('/').last();
        match name {
            Some(name) => name.to_string(),
            None => "attachment".to_string(),
        }
    };

    let attachment = AttachmentHandle {
        filename: name,
        content_type: Some(ct),
        url,
    };

    Some(attachment)
}
