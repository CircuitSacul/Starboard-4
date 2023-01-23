macro_rules! call_with_override_settings {
    ($macro_name: ident, $($extra_arg: expr),*) => {
        $macro_name!(
            $(
                $extra_arg,
            )*
            display_emoji,
            ping_author,
            use_server_profile,
            extra_embeds,
            use_webhook,
            color,
            jump_to_message,
            attachments_list,
            replied_to,
            required,
            required_remove,
            upvote_emojis,
            downvote_emojis,
            self_vote,
            allow_bots,
            require_image,
            older_than,
            newer_than,
            enabled,
            autoreact_upvote,
            autoreact_downvote,
            remove_invalid_reactions,
            link_deletes,
            link_edits,
            on_delete,
            cooldown_enabled,
            cooldown_count,
            cooldown_period,
            exclusive_group,
            exclusive_group_priority
        )
    };
}

pub(crate) use call_with_override_settings;
