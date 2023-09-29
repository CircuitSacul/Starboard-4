#[macro_export]
macro_rules! starboard_from_record {
    ($record: expr) => {{
        use $crate::{call_with_starboard_settings, settings_from_record};
        Starboard {
            id: $record.id,
            name: $record.name,
            channel_id: $record.channel_id,
            guild_id: $record.guild_id,
            webhook_id: $record.webhook_id,
            premium_locked: $record.premium_locked,
            settings: call_with_starboard_settings!(settings_from_record, $record),
        }
    }};
}

#[macro_export]
macro_rules! starboard_from_row {
    ($record: expr) => {{
        use sqlx::Row;
        use $crate::{call_with_starboard_settings, settings_from_row};
        Starboard {
            id: $record.get("id"),
            name: $record.get("name"),
            channel_id: $record.get("channel_id"),
            guild_id: $record.get("guild_id"),
            webhook_id: $record.get("webhook_id"),
            premium_locked: $record.get("premium_locked"),
            settings: call_with_starboard_settings!(settings_from_row, $record),
        }
    }};
}
