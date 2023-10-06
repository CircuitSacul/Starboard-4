use std::collections::HashSet;

use leptos::*;
use twilight_model::{
    guild::Guild,
    id::{marker::EmojiMarker, Id},
};

#[component]
pub fn MultiEmojiInput(
    id: &'static str,
    name: &'static str,
    initial: Vec<String>,
    guild: Guild,
) -> impl IntoView {
    let container_div_ref = create_node_ref::<html::Div>();
    let show_picker = move || {
        container_div_ref
            .get()
            .map(|elm| elm.style("display", "block"))
    };

    let emojis_to_str = |emojis: Vec<String>| emojis.into_iter().collect::<Vec<_>>().join(",");

    let value = create_rw_signal(emojis_to_str(initial));
    let emojis = create_memo(move |_| {
        let mut used = HashSet::new();
        value.with(|value| {
            value
                .split(',')
                .rev()
                .map(|s| s.to_owned())
                .filter(|e| !e.is_empty())
                .filter(|e| used.insert(e.to_owned()))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        })
    });

    let remove_emoji = move |emoji: &str| {
        value.set(
            emojis
                .get()
                .into_iter()
                .filter(|e| e != emoji)
                .collect::<Vec<_>>()
                .join(","),
        )
    };

    view! {
        <input
            id=id
            name=name
            type="hidden"
            prop:value=value
            on:change=move |e| value.set(event_target_value(&e))
        />
        <div class="flex flex-row">
            <For
                each=move || {emojis.get()}
                key=|e| e.to_owned()
                children=move |emoji| {
                    let emoji2 = emoji.clone();
                    view! {
                        <button
                            type="button"
                            class="btn btn-ghost btn-sm btn-square text-xl"
                            on:click=move |_| remove_emoji(&emoji)
                        >
                            <Emoji emoji=emoji2.into()/>
                        </button>
                    }
                }
            />

            <button
                type="button"
                class="btn btn-ghost btn-sm btn-square text-xl"
                on:click=move |_| {show_picker();}
            >
                +
            </button>
        </div>
        <EmojiPopup
            id=id
            container_div_ref=container_div_ref
            on_select=format!(
                r#"(emoji) => {{
                    console.log(emoji);
                    console.log({id}.value);
                    // cover every possibility cause you literally never know with js
                    if ({id}.value === null || {id}.value === undefined || {id}.value === "") {{
                        let newValue = emoji.native ? emoji.native : emoji.id;
                        {id}.value = newValue;
                        console.log(newValue, {id}.value);
                    }} else {{
                        let newValue = {id}.value + "," + (emoji.native ? emoji.native : emoji.id);
                        {id}.value = newValue;
                        console.log(newValue, {id}.value);
                    }}
                    var changeEvent = document.createEvent("HTMLEvents");
                    changeEvent.initEvent("change", true, false);
                    {id}.dispatchEvent(changeEvent);
                    picker_container_{id}.style.display = "none";
                }}"#
            )
            guild=guild
        />
    }
}

#[component]
pub fn EmojiButton<I: ToString>(
    id: &'static str,
    name: &'static str,
    initial: I,
    guild: Guild,
) -> impl IntoView {
    let value = create_rw_signal(initial.to_string());
    let container_div_ref = create_node_ref::<html::Div>();

    view! {
        <input
            id=id
            name=name
            type="hidden"
            prop:value=value
            on:change=move |e| value.set(event_target_value(&e))
        />
        <button
            type="button"
            id=format!("picker_button_{id}")
            class="btn btn-ghost btn-sm btn-square text-xl"
            on:click=move |_| {container_div_ref.get().map(|elm| elm.style("display", "block"));}
        >
            <Emoji emoji=value.into()/>
        </button>
        <EmojiPopup
            id=id
            container_div_ref=container_div_ref
            on_select=format!(
                r#"(emoji) => {{
                    console.log(emoji);
                    if (emoji.native !== undefined) {{
                        {id}.value = emoji.native;
                    }} else {{
                        {id}.value = emoji.id;
                    }}
                    var changeEvent = document.createEvent("HTMLEvents");
                    changeEvent.initEvent("change", true, false);
                    {id}.dispatchEvent(changeEvent);
                    picker_container_{id}.style.display = "none";
                }}"#
            )
            guild=guild
        />
    }
}

#[component]
pub fn Emoji(emoji: MaybeSignal<String>) -> impl IntoView {
    let emoji2 = emoji.clone();
    let custom = create_memo(move |_| {
        emoji2
            .get()
            .parse::<Id<EmojiMarker>>()
            .map(|id| format!("https://cdn.discordapp.com/emojis/{id}"))
            .ok()
    });

    view! {
        {move || {
            if let Some(custom) = custom.get() {
                view! {
                    <img
                        src=custom
                        style="max-width: 1em; max-height: 1em;"
                    />
                }.into_view()
            } else {
                emoji.get().into_view()
            }
        }}
    }
}

#[component]
pub fn EmojiPopup(
    id: &'static str,
    container_div_ref: NodeRef<html::Div>,
    on_select: String,
    guild: Guild,
) -> impl IntoView {
    let custom_id = guild.id.to_string();
    let emojis = guild.emojis;
    let emojis_ser = emojis
        .into_iter()
        .map(|emoji| {
            serde_json::json!({
                "name": emoji.name,
                "id": emoji.id.to_string(),
                "keywords": [emoji.name],
                "skins": [{
                    "src": format!("https://cdn.discordapp.com/emojis/{}", emoji.id)
                }]
            })
        })
        .collect::<Vec<_>>();
    let custom = serde_json::to_string(&emojis_ser).unwrap();
    let js = format!(
        r#"picker_container_{id}.replaceChildren(new EmojiMart.Picker({{
            set: 'twitter',
            custom: [{{
                id: "{custom_id}",
                name: "Custom",
                emojis: {custom},
            }}],
            maxFrequentRows: 0,
            onEmojiSelect: {on_select},
            onClickOutside: () => picker_container_{id}.style.display = "none"
        }}));"#
    );

    view! {
        <div
            ref=container_div_ref
            id=format!("picker_container_{id}")
            class="fixed"
            style="display: none"
        />
        <script>{js}</script>
    }
}
