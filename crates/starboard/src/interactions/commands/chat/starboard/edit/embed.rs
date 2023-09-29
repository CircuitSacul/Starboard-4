use twilight_interactions::command::{CommandModel, CreateCommand};

use database::Starboard;
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, parsing, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "embed",
    desc = "Edit the style of the embeds sent to your starboard."
)]
pub struct EditEmbedStyle {
    /// The starboard to edit.
    #[command(autocomplete = true)]
    name: String,

    /// The color of the embeds. Use 'none' for default.
    color: Option<String>,
    /// Whether to include a list of attachments.
    #[command(rename = "attachments-list")]
    attachments_list: Option<bool>,
    /// Whether to include the message that was replied to, if any.
    #[command(rename = "replied-to")]
    replied_to: Option<bool>,
}

impl EditEmbedStyle {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);

        let starboard = Starboard::get_by_name(&ctx.bot.db, &self.name, guild_id.get_i64()).await?;
        let mut starboard = match starboard {
            None => {
                ctx.respond_str("No starboard with that name was found.", true)
                    .await?;
                return Ok(());
            }
            Some(starboard) => starboard,
        };

        if let Some(val) = self.color {
            if val == "none" {
                starboard.settings.color = None;
            } else {
                match parsing::color::parse_color(&val) {
                    Ok(val) => starboard.settings.color = Some(val),
                    Err(why) => {
                        ctx.respond_str(why, true).await?;
                        return Ok(());
                    }
                }
            }
        }
        if let Some(val) = self.attachments_list {
            starboard.settings.attachments_list = val;
        }
        if let Some(val) = self.replied_to {
            starboard.settings.replied_to = val;
        }

        starboard.update_settings(&ctx.bot.db).await?;
        ctx.respond_str(
            &format!("Updated settings for starboard '{}'.", self.name),
            false,
        )
        .await?;
        Ok(())
    }
}
