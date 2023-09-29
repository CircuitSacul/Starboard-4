/// Get the guild_id from a CmdContext, sending an error to the user
/// if it is nonexistent (meaning the command was run inside DMs).
#[macro_export]
macro_rules! get_guild_id {
    ($ctx: expr) => {
        match $ctx.interaction.guild_id {
            None => {
                $ctx.respond_str("Please invite me to the server first.", true)
                    .await?;
                return Ok(());
            }
            Some(value) => value,
        }
    };
}
