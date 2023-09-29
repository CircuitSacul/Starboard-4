pub mod name;
pub mod regex;
pub mod relative_duration;
pub mod starboard_settings;

pub trait ToBotStr {
    fn to_bot_str(&self) -> String;
}
