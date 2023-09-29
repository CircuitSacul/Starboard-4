use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{ExclusiveGroup, StarboardOverride};
use errors::StarboardResult;

use crate::{
    get_guild_id,
    interactions::{commands::choices::on_delete::OnDelete, context::CommandCtx},
    parsing,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "behavior", desc = "Edit how the starboard should behave.")]
pub struct EditBehavior {
    /// The starboard to edit.
    #[command(autocomplete = true)]
    name: String,

    /// Whether the starboard is enabled.
    enabled: Option<bool>,
    /// Whether to automatically react to starboard messages with the upvote emojis.
    #[command(rename = "autoreact-upvote")]
    autoreact_upvote: Option<bool>,
    /// Whether to automatically react to starboard messages with the downvote emojis.
    #[command(rename = "autoreact-downvote")]
    autoreact_downvote: Option<bool>,
    /// Whether to remove reactions that don't meet requirements.
    #[command(rename = "remove-invalid-reactions")]
    remove_invalid_reactions: Option<bool>,
    /// If the original message is deleted, whether to also delete the starboard message.
    #[command(rename = "link-deletes")]
    link_deletes: Option<bool>,
    /// If the original message is edted, whether to also update the content of the starboard message.
    #[command(rename = "link-edits")]
    link_edits: Option<bool>,
    /// What to do if a moderator removes a post from the starboard manually.
    #[command(rename = "on-delete")]
    on_delete: Option<OnDelete>,
    /// Whether to enable the per-user vote cooldown.
    #[command(rename = "cooldown-enabled")]
    cooldown_enabled: Option<bool>,
    /// The size of the cooldown (e.x. "5/6" means 5 votes per 6 seconds).
    cooldown: Option<String>,
    /// Add this starboard to an exclusive group (only one at a time).
    #[command(rename = "exclusive-group", autocomplete = true)]
    exclusive_group: Option<String>,
    /// Remove this starboard from the exclusive group.
    #[command(rename = "remove-exclusive-group")]
    remove_exclusive_group: Option<bool>,
    #[command(rename = "exclusive-group-priority", min_value=-50, max_value=50)]
    /// Set the priority of this starboard in the exclusive group.
    exclusive_group_priority: Option<i64>,
}

impl EditBehavior {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();
        let ov = match StarboardOverride::get(&ctx.bot.db, guild_id, &self.name).await? {
            None => {
                ctx.respond_str("No override with that name was found.", true)
                    .await?;
                return Ok(());
            }
            Some(ov) => ov,
        };
        let mut settings = ov.get_overrides()?;

        if let Some(val) = self.enabled {
            settings.enabled = Some(val);
        }
        if let Some(val) = self.autoreact_upvote {
            settings.autoreact_upvote = Some(val);
        }
        if let Some(val) = self.autoreact_downvote {
            settings.autoreact_downvote = Some(val);
        }
        if let Some(val) = self.remove_invalid_reactions {
            settings.remove_invalid_reactions = Some(val);
        }
        if let Some(val) = self.link_deletes {
            settings.link_deletes = Some(val);
        }
        if let Some(val) = self.link_edits {
            settings.link_edits = Some(val);
        }
        if let Some(val) = self.on_delete {
            settings.on_delete = Some(val.value() as i16);
        }
        if let Some(val) = self.cooldown_enabled {
            settings.cooldown_enabled = Some(val);
        }
        if let Some(val) = self.cooldown {
            let (capacity, period) = match parsing::cooldown::parse_cooldown(&val) {
                Err(why) => {
                    ctx.respond_str(&why, true).await?;
                    return Ok(());
                }
                Ok(val) => val,
            };
            settings.cooldown_count = Some(capacity);
            settings.cooldown_period = Some(period);
        }
        if let Some(val) = self.exclusive_group {
            let group = ExclusiveGroup::get_by_name(&ctx.bot.db, guild_id, &val).await?;
            let Some(group) = group else {
                ctx.respond_str(
                    &format!(
                        concat!(
                    "Exclusive group '{}' does not exist. If you meant to remove the exclusive ",
                    "group, use `remove-exclusive-group: True` instead."
                ),
                        val
                    ),
                    true,
                )
                .await?;
                return Ok(());
            };
            settings.exclusive_group = Some(Some(group.id));
        }
        if let Some(val) = self.remove_exclusive_group {
            if val {
                settings.exclusive_group = Some(None);
            }
        }
        if let Some(val) = self.exclusive_group_priority {
            settings.exclusive_group_priority = Some(val as i16);
        }

        StarboardOverride::update_settings(&ctx.bot.db, ov.id, settings).await?;
        ctx.respond_str(
            &format!("Updated settings for override '{}'.", self.name),
            false,
        )
        .await?;

        Ok(())
    }
}
