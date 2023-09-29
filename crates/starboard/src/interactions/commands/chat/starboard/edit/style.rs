use twilight_interactions::command::{CommandModel, CreateCommand};

use database::Starboard;
use errors::StarboardResult;

use crate::{
    core::{
        emoji::{EmojiCommon, SimpleEmoji},
        starboard::webhooks::create_webhook,
    },
    get_guild_id,
    interactions::{commands::choices::go_to_message::GoToMessage, context::CommandCtx},
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "style", desc = "Edit the general style of your starboard.")]
pub struct EditGeneralStyle {
    /// The starboard to edit.
    #[command(autocomplete = true)]
    name: String,

    /// The emoji to show next to the point count. Use 'none' for nothing.
    #[command(rename = "display-emoji")]
    display_emoji: Option<String>,
    /// Whether to mention the author on starboard posts.
    #[command(rename = "ping-author")]
    ping_author: Option<bool>,
    /// Whether to use per-server avatar and nicknames for posts.
    #[command(rename = "use-server-profile")]
    use_server_profile: Option<bool>,
    /// Whether to include extra embeds that were on the original message.
    #[command(rename = "extra-embeds")]
    extra_embeds: Option<bool>,
    /// Where to put the "Go to Message" link.
    #[command(rename = "go-to-message")]
    go_to_message: Option<GoToMessage>,
    /// Whether to use a webhook for starboard messages.
    #[command(rename = "use-webhook")]
    use_webhook: Option<bool>,
}

impl EditGeneralStyle {
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

        if let Some(val) = self.display_emoji {
            let emoji = if val == "none" {
                None
            } else {
                let mut emojis = SimpleEmoji::from_user_input(&val, &ctx.bot, guild_id);
                if emojis.len() != 1 {
                    ctx.respond_str(
                        concat!(
                            "Please specify exactly one emoji for `display-emoji`, or use 'none' ",
                            "to remove."
                        ),
                        true,
                    )
                    .await?;
                    return Ok(());
                }

                emojis.pop()
            };
            starboard.settings.display_emoji = emoji.map(|emoji| emoji.into_stored());
        }
        if let Some(val) = self.ping_author {
            starboard.settings.ping_author = val;
        }
        if let Some(val) = self.use_server_profile {
            starboard.settings.use_server_profile = val;
        }
        if let Some(val) = self.extra_embeds {
            starboard.settings.extra_embeds = val;
        }
        if let Some(val) = self.go_to_message {
            starboard.settings.go_to_message = val.value() as i16;
        }
        let message;
        if let Some(val) = self.use_webhook {
            starboard.settings.use_webhook = val;

            message = Some(create_webhook(&ctx.bot, &starboard).await);
        } else {
            message = None;
        }

        let mut response = format!("Updated settings for '{}'.", self.name);
        if let Some(message) = message {
            response.push_str("\n\n");
            response.push_str(message);
        }

        starboard.update_settings(&ctx.bot.db).await?;
        ctx.respond_str(&response, false).await?;
        Ok(())
    }
}
