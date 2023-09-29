#[macro_export]
macro_rules! call_with_filters_settings {
    ($macro_name: ident, $($extra_arg: expr),*) => {
        $macro_name!(
            $(
                $extra_arg,
            )*
            instant_pass,
            instant_fail,
            user_has_all_of,
            user_has_some_of,
            user_missing_all_of,
            user_missing_some_of,
            user_is_bot,

            in_channel,
            not_in_channel,
            in_channel_or_sub_channels,
            not_in_channel_or_sub_channels,
            min_attachments,
            max_attachments,
            min_length,
            max_length,
            matches,
            not_matches,

            voter_has_all_of,
            voter_has_some_of,
            voter_missing_all_of,
            voter_missing_some_of,
            older_than,
            newer_than
        )
    }
}
