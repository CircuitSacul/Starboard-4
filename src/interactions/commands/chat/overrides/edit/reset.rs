use std::collections::HashSet;

use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    database::{helpers::settings::overrides::call_with_override_settings, StarboardOverride},
    errors::StarboardResult,
    get_guild_id,
    interactions::context::CommandCtx,
    unwrap_id,
};

macro_rules! reset_settings {
    ($overrides: expr, $reset: expr, $($setting: ident),*) => {{
        $(
            let setting = stringify!($setting);
            if $reset.contains(setting) {
                $overrides.$setting = None;
            }
        )*
    }}
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "reset",
    desc = "Reset override settings to the defaults used by the starboard."
)]
pub struct ResetOverrideSettings {
    /// The override to reset settings for.
    #[command(autocomplete = true)]
    name: String,
    /// The settings to reset, space seperated.
    reset: String,
}

impl ResetOverrideSettings {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = unwrap_id!(get_guild_id!(ctx));

        let ov = StarboardOverride::get(&ctx.bot.pool, guild_id, &self.name).await?;
        let ov = match ov {
            None => {
                ctx.respond_str("No override with that name was found.", true)
                    .await?;
                return Ok(());
            }
            Some(ov) => ov,
        };
        let mut settings = ov.get_overrides()?;

        let reset = self.reset.replace(',', " ").replace('-', "_");
        let mut reset: HashSet<_> = reset.split(' ').collect();
        let mut frontend_final_count_sub = 0;
        if reset.contains("cooldown")
            || reset.contains("cooldown_count")
            || reset.contains("cooldown_period")
        {
            reset.insert("cooldown_count");
            reset.insert("cooldown_period");
            reset.remove("cooldown");
            frontend_final_count_sub += 1;
        }

        call_with_override_settings!(reset_settings, settings, reset);

        StarboardOverride::update_settings(&ctx.bot.pool, ov.id, settings).await?;
        ctx.respond_str(
            &format!(
                "Reset {} setting(s) for override '{}'.",
                reset.len() - frontend_final_count_sub,
                ov.name
            ),
            false,
        )
        .await?;

        Ok(())
    }
}
