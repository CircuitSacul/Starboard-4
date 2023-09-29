use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component, Embed, MessageFlags,
    },
    id::{
        marker::{MessageMarker, UserMarker},
        Id,
    },
};

use errors::StarboardResult;

use crate::interactions::context::{CommandCtx, ComponentCtx};

use super::wait_for::wait_for_component;

pub fn components(current_page: usize, last_page: usize, done: bool) -> Vec<Component> {
    let buttons = vec![
        Component::Button(Button {
            custom_id: Some("paginator::to_start".to_string()),
            disabled: current_page == 0 || done,
            emoji: None,
            label: Some("<<".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("paginator::back".to_string()),
            disabled: current_page == 0 || done,
            emoji: None,
            label: Some("<".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("paginator::meta".to_string()),
            disabled: true,
            emoji: None,
            label: Some(format!("{}/{}", current_page + 1, last_page + 1)),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("paginator::next".to_string()),
            disabled: current_page == last_page || done,
            emoji: None,
            label: Some(">".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("paginator::to_end".to_string()),
            disabled: current_page == last_page || done,
            emoji: None,
            label: Some(">>".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
    ];

    let row = Component::ActionRow(ActionRow {
        components: buttons,
    });

    vec![row]
}

pub async fn simple(
    ctx: &mut CommandCtx,
    pages: Vec<(Option<String>, Option<Vec<Embed>>)>,
    user_id: Id<UserMarker>,
    ephemeral: bool,
) -> StarboardResult<()> {
    if pages.is_empty() {
        panic!("Paginator received no pages.");
    }

    let last_page = pages.len() - 1;

    let mut current_page: usize = 0;
    let mut message_id: Option<Id<MessageMarker>> = None;
    let mut btn_ctx: Option<ComponentCtx> = None;

    loop {
        // update the page
        let (raw_text, embeds) = &pages[current_page];
        let mut int_data = ctx
            .build_resp()
            .components(components(current_page, last_page, false));
        if let Some(raw_text) = raw_text {
            int_data = int_data.content(raw_text);
        }
        if let Some(embeds) = embeds {
            int_data = int_data.embeds(embeds.to_owned());
        }

        let msg_id = if let Some(msg_id) = message_id {
            btn_ctx.unwrap().edit(int_data.build()).await?;
            msg_id
        } else {
            if ephemeral {
                int_data = int_data.flags(MessageFlags::EPHEMERAL);
            }

            let msg = ctx.respond(int_data.build()).await?;
            let msg_id = msg.model().await?.id;
            message_id = Some(msg_id);
            msg_id
        };

        if pages.len() == 1 {
            return Ok(());
        }

        // wait for interactions
        btn_ctx = wait_for_component(
            ctx.bot.clone(),
            &[
                "paginator::to_start",
                "paginator::back",
                "paginator::next",
                "paginator::to_end",
            ],
            msg_id,
            user_id,
            60 * 5,
        )
        .await;

        if let Some(known_btn_ctx) = &btn_ctx {
            current_page = match &*known_btn_ctx.data.custom_id {
                "paginator::to_start" => 0,
                "paginator::back" => current_page - 1,
                "paginator::next" => current_page + 1,
                "paginator::to_end" => last_page,
                _ => unreachable!(),
            };
        } else {
            let i = ctx.bot.interaction_client().await;
            let mut update = i.update_response(&ctx.interaction.token);
            let comp = components(current_page, last_page, true);
            update = update.components(Some(&comp))?;
            update.await?;
            return Ok(());
        };
    }
}
