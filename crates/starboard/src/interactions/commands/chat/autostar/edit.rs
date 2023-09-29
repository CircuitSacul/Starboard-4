use twilight_interactions::command::{CommandModel, CreateCommand};

use database::AutoStarChannel;
use errors::StarboardResult;

use crate::{
    core::{
        emoji::{EmojiCommon, SimpleEmoji},
        premium::is_premium::is_guild_premium,
    },
    get_guild_id,
    interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "edit", desc = "Set the emojis for an autostar channel.")]
pub struct EditAutoStar {
    /// The name of the autostar channel to edit.
    #[command(autocomplete = true)]
    name: String,
    /// The emojis to use. Use "none" to set to none.
    emojis: Option<String>,
    /// The minimum number of characters a message needs.
    #[command(rename = "min-chars", min_value = 0, max_value = 5_000)]
    min_chars: Option<i64>,
    /// The maximum number of characters a message can have. Set to -1 to disable.
    #[command(rename = "max-chars", min_value = -1, max_value = 5_000)]
    max_chars: Option<i64>,
    /// Whether or not a message must include an image.
    #[command(rename = "require-image")]
    require_image: Option<bool>,
    /// Whether to delete messages that don't meet requirements.
    #[command(rename = "delete-invalid")]
    delete_invalid: Option<bool>,
}

impl EditAutoStar {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let asc = AutoStarChannel::get_by_name(&ctx.bot.db, &self.name, guild_id_i64).await?;
        let mut asc = match asc {
            None => {
                ctx.respond_str("No autostar channel with that name was found.", true)
                    .await?;
                return Ok(());
            }
            Some(asc) => asc,
        };

        let is_prem = is_guild_premium(&ctx.bot, guild_id_i64, true).await?;

        if let Some(val) = self.emojis {
            let emojis = SimpleEmoji::from_user_input(&val, &ctx.bot, guild_id).into_stored();
            if let Err(why) = asc.set_emojis(emojis, is_prem) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
        }
        if let Some(val) = self.min_chars {
            let min_chars = val as i16;
            if let Err(why) = asc.set_min_chars(min_chars) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
        }
        if let Some(val) = self.max_chars {
            let max_chars = if val == -1 { None } else { Some(val as i16) };
            if let Err(why) = asc.set_max_chars(max_chars) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
        }
        if let Some(val) = self.require_image {
            asc.require_image = val;
        }
        if let Some(val) = self.delete_invalid {
            asc.delete_invalid = val;
        }

        let asc = asc.update_settings(&ctx.bot.db).await?;

        if asc.is_none() {
            ctx.respond_str("No autostar channels with that name were found.", true)
                .await?;
            return Ok(());
        }

        // set the emojis
        ctx.respond_str(
            &format!("Updated the settings for autostar channel '{}'.", self.name),
            false,
        )
        .await?;

        Ok(())
    }
}
