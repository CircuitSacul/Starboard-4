use twilight_model::id::{marker::GuildMarker, Id};

use database::PermRole;

use crate::cache::Cache;

use super::into_id::IntoId;

pub trait SortVecPermRole {
    fn sort_permroles(&mut self, cache: &Cache);
}

impl SortVecPermRole for Vec<PermRole> {
    fn sort_permroles(&mut self, cache: &Cache) {
        let guild_id: Id<GuildMarker> = match self.first() {
            Some(pr) => pr.guild_id.into_id(),
            None => return,
        };

        cache.guilds.with(&guild_id, |_, guild| {
            let guild = match guild {
                None => return,
                Some(guild) => guild,
            };

            self.sort_by_key(|pr| guild.roles.get(&pr.role_id.into_id()).map(|r| r.position));
        })
    }
}
