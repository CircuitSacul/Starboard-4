use database::validation::{
    starboard_settings::{validate_required, validate_required_remove},
    ToBotStr,
};

use super::none_or;

pub fn parse_number(val: &str) -> Result<Option<i16>, String> {
    match none_or::<i16>(val) {
        Err(_) => Err(format!("I couldn't interpret {val} as a number.")),
        Ok(v) => Ok(v),
    }
}

pub fn parse_required(val: &str, remove: Option<i16>) -> Result<Option<i16>, String> {
    let val = parse_number(val)?;
    match val {
        Some(val) => validate_required(val, remove)
            .map_err(|e| e.to_bot_str())
            .map(Some),
        None => Ok(None),
    }
}

pub fn parse_required_remove(val: &str, required: Option<i16>) -> Result<Option<i16>, String> {
    let val = parse_number(val)?;
    match val {
        Some(val) => validate_required_remove(val, required)
            .map_err(|e| e.to_bot_str())
            .map(Some),
        None => Ok(None),
    }
}
