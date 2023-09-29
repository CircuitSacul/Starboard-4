#[macro_export]
macro_rules! call_with_autostar_settings {
    ($macro_name: ident, $($extra_arg: expr),*) => {
        $macro_name!(
            $(
                $extra_arg,
            )*
            emojis,
            min_chars,
            max_chars,
            require_image,
            delete_invalid
        )
    };
}
