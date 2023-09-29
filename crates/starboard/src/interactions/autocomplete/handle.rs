use twilight_model::{
    application::interaction::application_command::{CommandDataOption, CommandOptionValue},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use errors::StarboardResult;

use crate::interactions::context::CommandCtx;

use super::{
    autoredeem::autoredeem_autocomplete, autostar_name::autostar_name_autocomplete,
    exclusive_group_name::exclusive_group_name_autocomplete,
    filter_group::filter_group_name_autocomplete, override_name::override_name_autocomplete,
    starboard_name::starboard_name_autocomplete,
};

pub fn get_sub_options(options: &Vec<CommandDataOption>) -> Option<&Vec<CommandDataOption>> {
    if options.len() != 1 {
        return None;
    }

    match &options[0].value {
        CommandOptionValue::SubCommand(options) => Some(options),
        CommandOptionValue::SubCommandGroup(options) => Some(options),
        _ => None,
    }
}

fn parse(ctx: &CommandCtx) -> (String, &str) {
    let mut name = ctx.data.name.clone();
    let options;

    if let Some(sub_options) = get_sub_options(&ctx.data.options) {
        let sub = &ctx.data.options[0];
        name.push(' ');
        name.push_str(&sub.name);
        if let Some(subsub_options) = get_sub_options(sub_options) {
            let subcommand = &sub_options[0];
            name.push(' ');
            name.push_str(&subcommand.name);
            options = subsub_options;
        } else {
            options = sub_options;
        }
    } else {
        options = &ctx.data.options;
    }

    for option in options {
        if let CommandOptionValue::Focused(val, _) = &option.value {
            name.push(' ');
            name.push_str(&option.name);
            return (name, val);
        }
    }

    unreachable!("No focused option in autocomplete response.");
}

pub async fn handle_autocomplete(ctx: CommandCtx) -> StarboardResult<()> {
    let (qual_name, focused) = parse(&ctx);
    let options = match qual_name.as_str() {
        // misc
        "random starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        "moststarred starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        "utils force starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        "utils unforce starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        // premium
        "premium autoredeem disable server" => autoredeem_autocomplete(&ctx, focused).await?,
        "premium-locks move-autostar from" => autostar_name_autocomplete(&ctx, focused).await?,
        "premium-locks move-autostar to" => autostar_name_autocomplete(&ctx, focused).await?,
        "premium-locks move-starboard from" => starboard_name_autocomplete(&ctx, focused).await?,
        "premium-locks move-starboard to" => starboard_name_autocomplete(&ctx, focused).await?,
        // autostar channels
        "autostar delete name" => autostar_name_autocomplete(&ctx, focused).await?,
        "autostar view name" => autostar_name_autocomplete(&ctx, focused).await?,
        "autostar edit name" => autostar_name_autocomplete(&ctx, focused).await?,
        "autostar rename current-name" => autostar_name_autocomplete(&ctx, focused).await?,
        "autostar filters add autostar-channel" => {
            autostar_name_autocomplete(&ctx, focused).await?
        }
        "autostar filters add filter-group" => {
            filter_group_name_autocomplete(&ctx, focused).await?
        }
        "autostar filters remove autostar-channel" => {
            autostar_name_autocomplete(&ctx, focused).await?
        }
        "autostar filters remove filter-group" => {
            filter_group_name_autocomplete(&ctx, focused).await?
        }
        // starboards
        "starboards delete name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards view name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards edit embed name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards edit style name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards edit requirements name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards edit behavior name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards edit behavior exclusive-group" => {
            exclusive_group_name_autocomplete(&ctx, focused).await?
        }
        "starboards rename current-name" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards filters add starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards filters add filter-group" => {
            filter_group_name_autocomplete(&ctx, focused).await?
        }
        "starboards filters remove starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        "starboards filters remove filter-group" => {
            filter_group_name_autocomplete(&ctx, focused).await?
        }
        // overrides
        "overrides create starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        "overrides delete name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides rename current-name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides channels set override" => override_name_autocomplete(&ctx, focused).await?,
        "overrides channels remove override" => override_name_autocomplete(&ctx, focused).await?,
        "overrides channels add override" => override_name_autocomplete(&ctx, focused).await?,
        "overrides edit requirements name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides edit behavior name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides edit behavior exclusive-group" => {
            exclusive_group_name_autocomplete(&ctx, focused).await?
        }
        "overrides edit style name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides edit embed name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides edit reset name" => override_name_autocomplete(&ctx, focused).await?,
        "overrides view name" => override_name_autocomplete(&ctx, focused).await?,
        // exclusive groups
        "exclusive-groups delete name" => exclusive_group_name_autocomplete(&ctx, focused).await?,
        "exclusive-groups rename original-name" => {
            exclusive_group_name_autocomplete(&ctx, focused).await?
        }
        // filter groups
        "filters create-filter group" => filter_group_name_autocomplete(&ctx, focused).await?,
        "filters delete-filter group" => filter_group_name_autocomplete(&ctx, focused).await?,
        "filters move-filter group" => filter_group_name_autocomplete(&ctx, focused).await?,
        "filters rename-group current-name" => {
            filter_group_name_autocomplete(&ctx, focused).await?
        }
        "filters delete-group name" => filter_group_name_autocomplete(&ctx, focused).await?,
        "filters view group" => filter_group_name_autocomplete(&ctx, focused).await?,
        "filters edit group" => filter_group_name_autocomplete(&ctx, focused).await?,
        // permroles
        "permroles edit-starboard starboard" => starboard_name_autocomplete(&ctx, focused).await?,
        qual => todo!("Unexpected autocomplete for {}.", qual),
    };

    let i = ctx.bot.interaction_client().await;
    let data = InteractionResponseDataBuilder::new()
        .choices(options)
        .build();
    let resp = InteractionResponse {
        data: Some(data),
        kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
    };
    i.create_response(ctx.interaction.id, &ctx.interaction.token, &resp)
        .await?;

    Ok(())
}
