use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::{
    component::{ActionRow, Button, ButtonStyle},
    Component,
};
use twilight_util::builder::embed::EmbedFieldBuilder;

use common::constants;
use errors::StarboardResult;

use crate::{concat_format, interactions::context::CommandCtx, utils::embed};

fn buttons() -> Vec<Component> {
    let link_btn = |name: &str, link: &str| {
        Component::Button(Button {
            custom_id: None,
            disabled: false,
            emoji: None,
            label: Some(name.into()),
            style: ButtonStyle::Link,
            url: Some(link.into()),
        })
    };

    let buttons = vec![
        link_btn("Invite", constants::INVITE_URL),
        link_btn("Support", constants::SUPPORT_URL),
        link_btn("Documentation", constants::DOCS_URL),
        link_btn("Source", constants::SOURCE_URL),
        link_btn("Premium", constants::PATREON_URL),
    ];

    let row = Component::ActionRow(ActionRow {
        components: buttons,
    });

    vec![row]
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "help", desc = "Get help with and general info for Starboard.")]
pub struct Help;

impl Help {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let emb = embed::build()
            .title("Starboard - The Best Discord Starboard")
            .description("Starboard is a reliable and feature-rich starboard bot for Discord.")
            .field(EmbedFieldBuilder::new(
                "Useful Commands",
                concat!(
                    "`/starboards view`: View all of your starboards.\n",
                    "`/starboards create`: Create a new starboard.\n",
                    "`/starboards edit`: Edit the configuration for a starboard.\n",
                ),
            ))
            .field(EmbedFieldBuilder::new(
                "Starboard's Features",
                concat!(
                    "Starboard's key free features are:\n",
                    "- Multiple starboards\n",
                    "- Custom avatar/username for starboards (via webhooks)\n",
                    "- Autostar channels\n",
                    "- Complete per-channel starboard configuration\n",
                    "- Limited per-role starboard configuration\n",
                ),
            ))
            .field(EmbedFieldBuilder::new(
                "Support Starboard",
                concat_format!(
                    "If you like Starboard and want to support it, you can do ";
                    "so by [voting]({}) or " <- constants::VOTE_URL;
                    "[leaving a review]({}) " <- constants::REVIEW_URL;
                    "on Top.GG.\n\nIf you really want, you can also get Starboard Premium ";
                    "by [becoming a patron]({})." <- constants::PATREON_URL;
                ),
            ))
            .field(EmbedFieldBuilder::new(
                "Starboard Premium",
                concat_format!(
                    "Premium works using a credit-based system. Each USD that you spend is ";
                    "equivalent to 1 premium credit. Once you have 3 credits, you can redeem it ";
                    "for 1 month of premium in any server.\n\n";
                    "Use `/premium credits` to see your credits, and `/premium redeem` to redeem ";
                    "premium in a server.\n\n";
                    "To get premium, visit [patreon.com]({})." <- constants::PATREON_URL;
                ),
            ))
            .build();

        ctx.respond(ctx.build_resp().embeds([emb]).components(buttons()).build())
            .await?;

        Ok(())
    }
}
