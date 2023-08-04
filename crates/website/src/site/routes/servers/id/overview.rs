use leptos::*;

#[component]
pub fn Overview(cx: Scope) -> impl IntoView {
    let guild = expect_context::<super::GuildContext>(cx);

    let content = move || format!("{:?}", guild.read(cx));
    view! { cx, <Suspense fallback=|| view! { cx, "Loading..." }>{content}</Suspense> }
}
