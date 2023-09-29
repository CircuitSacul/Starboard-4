use twilight_mention::Mention;
use twilight_model::id::{
    marker::{GuildMarker, RoleMarker},
    Id,
};

use crate::client::bot::StarboardBot;

pub fn get_deleted_roles(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    role_ids: impl Iterator<Item = Id<RoleMarker>>,
) -> (String, Vec<Id<RoleMarker>>) {
    let to_delete = bot.cache.guilds.with(&guild_id, |_, guild| {
        let guild = guild.as_ref().unwrap();
        role_ids
            .filter(|r| !guild.roles.contains_key(r))
            .collect::<Vec<_>>()
    });

    (
        to_delete
            .iter()
            .map(|r| r.mention().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        to_delete,
    )
}
