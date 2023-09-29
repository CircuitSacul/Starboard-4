use twilight_interactions::command::{CommandModel, CreateCommand};

use database::StarboardOverride;
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, parsing, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "embed",
    desc = "Edit the style of the embeds sent to your starboard."
)]
pub struct EditEmbedStyle {
    /// The override to edit.
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

        let ov = StarboardOverride::get(&ctx.bot.db, guild_id.get_i64(), &self.name).await?;
        let ov = match ov {
            None => {
                ctx.respond_str("No override with that name was found.", true)
                    .await?;
                return Ok(());
            }
            Some(ov) => ov,
        };
        let mut settings = ov.get_overrides()?;

        if let Some(val) = self.color {
            if val == "none" {
                settings.color = Some(None);
            } else {
                match parsing::color::parse_color(&val) {
                    Ok(val) => settings.color = Some(Some(val)),
                    Err(why) => {
                        ctx.respond_str(why, true).await?;
                        return Ok(());
                    }
                }
            }
        }
        if let Some(val) = self.attachments_list {
            settings.attachments_list = Some(val);
        }
        if let Some(val) = self.replied_to {
            settings.replied_to = Some(val);
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
