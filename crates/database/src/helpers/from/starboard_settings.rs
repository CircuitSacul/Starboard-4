#[macro_export]
macro_rules! settings_from_record {
    ($has_settings: expr, $($name: ident),*) => {{
        use $crate::StarboardSettings;
        StarboardSettings {
            $(
                $name: $has_settings.$name,
            )*
        }
    }};
}

#[macro_export]
macro_rules! settings_from_row {
    ($has_settings: expr, $($name: ident),*) => {{
        use $crate::StarboardSettings;
        StarboardSettings {
            $(
                $name: $has_settings.get(stringify!($name)),
            )*
        }
    }};
}
