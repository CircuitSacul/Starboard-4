use lazy_static::lazy_static;
use twilight_model::channel::message::{embed::EmbedImage, Embed};

use super::AttachmentHandle;

pub enum ImgurResult {
    Video(AttachmentHandle),
    Image(Box<Embed>),
}

pub fn modify_imgur_embed(mut embed: Embed) -> ImgurResult {
    if let Some(video) = &embed.video {
        if let Some(proxy) = &video.proxy_url {
            let ext = proxy.split('.').last().unwrap_or("mp4");
            return ImgurResult::Video(AttachmentHandle {
                filename: format!("imgur_video.{ext}"),
                content_type: Some("video".to_string()),
                url: proxy.to_owned(),
            });
        }
    }

    let thumb = std::mem::take(&mut embed.thumbnail);
    if let Some(thumb) = thumb {
        if let Some(url) = modify_imgur_url(&thumb.url) {
            embed.image = Some(EmbedImage {
                height: None,
                width: None,
                proxy_url: None,
                url,
            });
        }
    }

    ImgurResult::Image(Box::new(embed))
}

pub fn modify_imgur_url(url: &str) -> Option<String> {
    lazy_static! {
        static ref RE: regex::Regex =
            regex::Regex::new(r"https://i\.imgur\.com/(\w+)\.(\w+)").unwrap();
    }

    let caps: Vec<_> = RE.captures_iter(url).collect();
    let groups = caps.get(0)?;

    let id = &groups[1];
    let ext = &groups[2];
    let id = id.strip_suffix('h').unwrap_or(id);

    Some(format!("https://i.imgur.com/{id}.{ext}"))
}
