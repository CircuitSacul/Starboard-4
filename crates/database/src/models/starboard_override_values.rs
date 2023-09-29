use serde::{Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;

fn null_to_some_none<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let val: Option<T> = Deserialize::deserialize(deserializer)?;

    Ok(Some(val))
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OverrideValues {
    // General Style
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub display_emoji: Option<Option<String>>,
    pub ping_author: Option<bool>,
    pub use_server_profile: Option<bool>,
    pub extra_embeds: Option<bool>,
    pub use_webhook: Option<bool>,

    // Embed Style
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub color: Option<Option<i32>>,
    pub go_to_message: Option<i16>,
    pub attachments_list: Option<bool>,
    pub replied_to: Option<bool>,

    // Requirements
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub required: Option<Option<i16>>,
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub required_remove: Option<Option<i16>>,
    pub upvote_emojis: Option<Vec<String>>,
    pub downvote_emojis: Option<Vec<String>>,
    pub self_vote: Option<bool>,
    pub allow_bots: Option<bool>,
    pub require_image: Option<bool>,
    pub older_than: Option<i64>,
    pub newer_than: Option<i64>,
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub matches: Option<Option<String>>,
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub not_matches: Option<Option<String>>,

    // Behavior
    pub enabled: Option<bool>,
    pub autoreact_upvote: Option<bool>,
    pub autoreact_downvote: Option<bool>,
    pub remove_invalid_reactions: Option<bool>,
    pub link_deletes: Option<bool>,
    pub link_edits: Option<bool>,
    pub on_delete: Option<i16>,
    pub cooldown_enabled: Option<bool>,
    pub cooldown_count: Option<i16>,
    pub cooldown_period: Option<i16>,
    #[serde(deserialize_with = "null_to_some_none", default)]
    pub exclusive_group: Option<Option<i32>>,
    pub exclusive_group_priority: Option<i16>,
}
