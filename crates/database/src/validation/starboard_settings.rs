//! Validation for certain starboard settings that are shared between
//! starboards and overrides, but not elsewhere and thus don't deserve
//! their own file.

use std::collections::HashSet;

use common::constants;

use super::ToBotStr;

pub enum RequiredErr {
    LessThanRemove,
    TooSmall,
    TooLarge,
}

impl ToBotStr for RequiredErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::LessThanRemove => "`required` must be greater than `required-remove`.".into(),
            Self::TooSmall => format!(
                "`required` cannot be less than {}.",
                constants::MIN_REQUIRED
            ),
            Self::TooLarge => format!(
                "`required` cannot be greater than {}.",
                constants::MAX_REQUIRED
            ),
        }
    }
}

pub fn validate_required(val: i16, required_remove: Option<i16>) -> Result<i16, RequiredErr> {
    if let Some(required_remove) = required_remove {
        if val <= required_remove {
            return Err(RequiredErr::LessThanRemove);
        }
    }

    if val < constants::MIN_REQUIRED {
        Err(RequiredErr::TooSmall)
    } else if val > constants::MAX_REQUIRED {
        Err(RequiredErr::TooLarge)
    } else {
        Ok(val)
    }
}

pub enum RemoveErr {
    GreaterThanRequired,
    TooSmall,
    TooLarge,
}

impl ToBotStr for RemoveErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::GreaterThanRequired => "`required-remove` must be less than `required.`".into(),
            Self::TooSmall => format!(
                "`required-remove` cannot be less than {}.",
                constants::MIN_REQUIRED_REMOVE
            ),
            Self::TooLarge => format!(
                "`required-remove` cannot be greater than {}.",
                constants::MAX_REQUIRED_REMOVE
            ),
        }
    }
}

pub fn validate_required_remove(val: i16, required: Option<i16>) -> Result<i16, RemoveErr> {
    if let Some(required) = required {
        if val >= required {
            return Err(RemoveErr::GreaterThanRequired);
        }
    }

    if val < constants::MIN_REQUIRED_REMOVE {
        Err(RemoveErr::TooSmall)
    } else if val > constants::MAX_REQUIRED_REMOVE {
        Err(RemoveErr::TooLarge)
    } else {
        Ok(val)
    }
}

pub enum XPMulErr {
    TooLarge,
    TooSmall,
}

impl ToBotStr for XPMulErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::TooLarge => format!(
                "`xp-multiplier` cannot be greater than {}.",
                constants::MAX_XP_MULTIPLIER
            ),
            Self::TooSmall => format!(
                "`xp-multiplier` cannot be less than {}.",
                constants::MIN_XP_MULTIPLIER
            ),
        }
    }
}

pub fn validate_xp_multiplier(val: f32) -> Result<(), XPMulErr> {
    if val > constants::MAX_XP_MULTIPLIER {
        Err(XPMulErr::TooLarge)
    } else if val < constants::MIN_XP_MULTIPLIER {
        Err(XPMulErr::TooSmall)
    } else {
        Ok(())
    }
}

pub enum CooldownErr {
    Negative,
    CapacityTooLarge,
    PeriodTooLarge,
}

impl ToBotStr for CooldownErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::Negative => {
                "The capacity and period for the cooldown must be greater than 0.".into()
            }
            Self::CapacityTooLarge => format!(
                "The cooldown capacity cannot be greater than {}.",
                constants::MAX_COOLDOWN_CAPACITY
            ),
            Self::PeriodTooLarge => format!(
                "The cooldown period cannot be greater than {}.",
                constants::MAX_COOLDOWN_PERIOD
            ),
        }
    }
}

pub fn validate_cooldown(capacity: i16, period: i16) -> Result<(), CooldownErr> {
    if capacity <= 0 || period <= 0 {
        Err(CooldownErr::Negative)
    } else if capacity > constants::MAX_COOLDOWN_CAPACITY {
        Err(CooldownErr::CapacityTooLarge)
    } else if period > constants::MAX_COOLDOWN_PERIOD {
        Err(CooldownErr::PeriodTooLarge)
    } else {
        Ok(())
    }
}

pub enum VoteEmojiErr {
    EmojisNotUnique,
    LimitReached,
    PremiumLimitReached,
}

impl ToBotStr for VoteEmojiErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::EmojisNotUnique => {
                "`upvote-emojis` and `downvote-emojis` cannot share emojis.".into()
            }
            Self::LimitReached => format!(
                "You can only have up to {} vote emojis. Upgrade to premium to get up to {}.",
                constants::MAX_VOTE_EMOJIS,
                constants::MAX_PREM_VOTE_EMOJIS
            ),
            Self::PremiumLimitReached => format!(
                "You can only have up to {} vote emojis.",
                constants::MAX_PREM_VOTE_EMOJIS
            ),
        }
    }
}

/// assumes that the actual contents of upvote and downvote
/// are valid emojis.
pub fn validate_vote_emojis(
    upvote: &[String],
    downvote: &[String],
    premium: bool,
) -> Result<(), VoteEmojiErr> {
    let unique_upvote: HashSet<_> = upvote.iter().collect();
    let unique_downvote: HashSet<_> = downvote.iter().collect();

    if unique_upvote
        .intersection(&unique_downvote)
        .next()
        .is_some()
    {
        return Err(VoteEmojiErr::EmojisNotUnique);
    }

    let limit = if premium {
        constants::MAX_PREM_VOTE_EMOJIS
    } else {
        constants::MAX_VOTE_EMOJIS
    };

    if unique_upvote.len() + unique_downvote.len() > limit {
        let err = match premium {
            false => VoteEmojiErr::LimitReached,
            true => VoteEmojiErr::PremiumLimitReached,
        };
        return Err(err);
    }

    Ok(())
}
