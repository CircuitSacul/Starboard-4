use common::constants;

use super::ToBotStr;

pub enum RegexErr {
    NotPremium,
    TooLong,
    ParseError(regex::Error),
}

impl ToBotStr for RegexErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::NotPremium => "The `matches` and `not-matches` settings require premium".into(),
            Self::TooLong => format!(
                "The `matches` and `not-matches` settings must be under {} characters.",
                constants::MAX_REGEX_LENGTH
            ),
            Self::ParseError(err) => format!("```\n{err}\n```"),
        }
    }
}

pub fn validate_regex(input: String, is_premium: bool) -> Result<Option<String>, RegexErr> {
    if !is_premium {
        return Err(RegexErr::NotPremium);
    }

    if input.len() > constants::MAX_REGEX_LENGTH as usize {
        return Err(RegexErr::TooLong);
    }

    if input == ".*" {
        Ok(None)
    } else {
        match regex::Regex::new(&input) {
            Ok(_) => Ok(Some(input)),
            Err(why) => Err(RegexErr::ParseError(why)),
        }
    }
}
