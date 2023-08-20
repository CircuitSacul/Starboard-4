pub mod add;
mod api;
pub mod id;

use leptos::*;
use leptos_icons::*;
use leptos_router::*;

use crate::site::components::{toast, Card, CardList, CardSkeleton, Toast, ToastedSusp};

use super::GuildIdContext;

pub type CreateStarboardAction = Action<self::api::CreateStarboard, Result<(), ServerFnError>>;

#[component]
pub fn Starboards(cx: Scope) -> impl IntoView {
    let create_sb: CreateStarboardAction = create_server_action::<self::api::CreateStarboard>(cx);
    provide_context(cx, create_sb);

    let guild_id = expect_context::<GuildIdContext>(cx);

    let starboards = create_resource(
        cx,
        move || (guild_id.get(), (create_sb.version().get(),)),
        move |(guild_id, _)| async move {
            let Some(guild_id) = guild_id else {
                return Err(ServerFnError::ServerError("No guild ID.".to_string()));
            };
            self::api::get_starboards(cx, guild_id).await
        },
    );

    create_effect(cx, move |_| {
        if let Some(Err(why)) = create_sb.value().get() {
            toast(cx, Toast::error(why))
        }
    });

    let starboards_view = move |cx| {
        starboards.read(cx).map(|sb| {
            sb.map(|sb| {
                let sb = store_value(cx, sb);
                view! { cx,
                    <For
                        each=move || sb.with_value(|sb| sb.clone())
                        key=|sb| sb.0
                        view=move |cx, sb| {
                            view! { cx, <Card title=sb.1.name href=sb.0.to_string()/> }
                        }
                    />
                }
            })
            .map_err(|e| e.clone())
        })
    };

    view! { cx,
        <Outlet/>
        <CardList>
            <div class="flex justify-end">
                <A href="add" class="btn btn-outline">
                    <Icon icon=crate::icon!(FaPlusSolid)/>
                    "New Starboard"
                </A>
            </div>

            <ToastedSusp fallback=move || {
                view! { cx,
                    <For each=|| 0..10 key=|t| *t view=move |_, _| view! { cx, <CardSkeleton/> }/>
                }
            }>{move || starboards_view(cx)}</ToastedSusp>
        </CardList>
    }
}
