use twilight_model::channel::message::{embed::EmbedImage, Embed};

pub fn modify_yt_embed(embed: &mut Embed) {
    embed.description = None;

    if let Some(thumb) = std::mem::take(&mut embed.thumbnail) {
        embed.image = Some(EmbedImage {
            height: None,
            width: None,
            proxy_url: None,
            url: thumb.url,
        });
    }
}
