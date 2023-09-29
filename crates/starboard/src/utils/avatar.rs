use twilight_model::{
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    util::ImageHash,
};

pub trait ImageHashAvatar {
    fn global_avatar(&self, user_id: Id<UserMarker>) -> String;
    fn guild_avatar(&self, user_id: Id<UserMarker>, guild_id: Id<GuildMarker>) -> String;
}

impl ImageHashAvatar for ImageHash {
    fn global_avatar(&self, user_id: Id<UserMarker>) -> String {
        format!("https://cdn.discordapp.com/avatars/{user_id}/{self}.png")
    }

    fn guild_avatar(&self, user_id: Id<UserMarker>, guild_id: Id<GuildMarker>) -> String {
        format!("https://cdn.discordapp.com/guilds/{guild_id}/users/{user_id}/avatars/{self}.png")
    }
}
