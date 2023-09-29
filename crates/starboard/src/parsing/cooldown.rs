use database::validation::{starboard_settings::validate_cooldown, ToBotStr};
use lazy_static::lazy_static;

pub fn parse_cooldown(inp: &str) -> Result<(i16, i16), String> {
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"(?P<count>\d+).+?(?P<secs>\d+)").unwrap();
    }

    let found = match RE.captures(inp) {
        None => {
            return Err(concat!(
                "I couldn't parse the cooldown you passed. The ",
                "correct format is `capacity/period` (e.x. `5/6`)."
            )
            .to_string())
        }
        Some(found) => found,
    };

    let capacity = found.name("count").unwrap().as_str();
    let capacity: i16 = match capacity.parse() {
        Err(_) => return Err(format!("{capacity} is not a valid number.")),
        Ok(capacity) => capacity,
    };
    let period = found.name("secs").unwrap().as_str();
    let period: i16 = match period.parse() {
        Err(_) => return Err(format!("{period} is not a valid number.")),
        Ok(period) => period,
    };

    validate_cooldown(capacity, period).map_err(|e| e.to_bot_str())?;
    Ok((capacity, period))
}
