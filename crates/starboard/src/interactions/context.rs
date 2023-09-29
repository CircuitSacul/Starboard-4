use std::sync::Arc;

use twilight_http::Response;
use twilight_model::{
    application::interaction::{
        application_command::CommandData, message_component::MessageComponentInteractionData,
        Interaction,
    },
    channel::{
        message::{AllowedMentions, MessageFlags},
        Message,
    },
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use errors::StarboardResult;

use crate::client::bot::StarboardBot;

pub type CommandCtx = Ctx<CommandData>;
pub type ComponentCtx = Ctx<MessageComponentInteractionData>;

#[derive(Debug)]
pub struct Ctx<T> {
    pub bot: Arc<StarboardBot>,
    pub interaction: Interaction,
    pub data: T,
    responded: bool,
}

type TwResult = StarboardResult<Response<Message>>;

impl<T> Ctx<T> {
    pub fn new(bot: Arc<StarboardBot>, interaction: Interaction, data: T) -> Self {
        Self {
            bot,
            interaction,
            data,
            responded: false,
        }
    }

    pub fn build_resp(&self) -> InteractionResponseDataBuilder {
        InteractionResponseDataBuilder::new().allowed_mentions(AllowedMentions::default())
    }

    pub async fn raw_respond(
        &mut self,
        data: Option<InteractionResponseData>,
        kind: InteractionResponseType,
    ) -> TwResult {
        let i = self.bot.interaction_client().await;

        if self.responded {
            let mut followup = i.create_followup(&self.interaction.token);
            let data = match data {
                None => panic!("cannot followup without data"),
                Some(data) => data,
            };

            if let Some(mentions) = &data.allowed_mentions {
                followup = followup.allowed_mentions(Some(mentions));
            }
            if let Some(attachments) = &data.attachments {
                followup = followup.attachments(attachments)?;
            }
            if let Some(components) = &data.components {
                followup = followup.components(components)?;
            }
            if let Some(content) = &data.content {
                followup = followup.content(content)?;
            }
            if let Some(embeds) = &data.embeds {
                followup = followup.embeds(embeds)?;
            }
            if let Some(flags) = data.flags {
                followup = followup.flags(flags);
            }
            if let Some(tts) = data.tts {
                followup = followup.tts(tts);
            }

            followup.await.map_err(|e| e.into())
        } else {
            i.create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse { data, kind },
            )
            .await?;

            self.responded = true;

            i.response(&self.interaction.token)
                .await
                .map_err(|e| e.into())
        }
    }

    pub async fn defer(&mut self, ephemeral: bool) -> TwResult {
        let mut data = self.build_resp();
        if ephemeral {
            data = data.flags(MessageFlags::EPHEMERAL);
        }

        self.raw_respond(
            Some(data.build()),
            InteractionResponseType::DeferredChannelMessageWithSource,
        )
        .await
    }

    pub async fn respond(&mut self, data: InteractionResponseData) -> TwResult {
        self.raw_respond(
            Some(data),
            InteractionResponseType::ChannelMessageWithSource,
        )
        .await
    }

    pub async fn respond_str(&mut self, response: &str, ephemeral: bool) -> TwResult {
        let mut data = self.build_resp().content(response);
        if ephemeral {
            data = data.flags(MessageFlags::EPHEMERAL);
        }

        self.raw_respond(
            Some(data.build()),
            InteractionResponseType::ChannelMessageWithSource,
        )
        .await
    }

    pub async fn edit(&mut self, data: InteractionResponseData) -> TwResult {
        self.raw_respond(Some(data), InteractionResponseType::UpdateMessage)
            .await
    }

    pub async fn edit_str(&mut self, response: &str, clear_comps: bool) -> TwResult {
        let mut data = self.build_resp().content(response);
        if clear_comps {
            data = data.components([]);
        }

        self.edit(data.build()).await
    }
}
