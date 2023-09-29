use std::time::Duration;

use common::constants;

use super::ToBotStr;

#[derive(Debug, Clone, Copy)]
pub enum RelativeDurationErr {
    OlderThanGreaterThanNewerThan,
    OlderThanNegative,
    NewerThanNegative,
    OlderThanTooLarge,
    NewerThanTooLarge,
}

impl ToBotStr for RelativeDurationErr {
    fn to_bot_str(&self) -> String {
        match self {
            Self::OlderThanGreaterThanNewerThan => {
                "`older-than` must be less than `newer-than`".into()
            }
            Self::OlderThanNegative => "`older-than` must be positive.".into(),
            Self::NewerThanNegative => "`newer-than` must be positive.".into(),
            Self::OlderThanTooLarge => format!(
                "`older-than` cannot be greater than `{}`.",
                humantime::format_duration(Duration::from_secs(constants::MAX_OLDER_THAN as u64))
            ),
            Self::NewerThanTooLarge => format!(
                "`newer-than` cannot be greater than `{}`.",
                humantime::format_duration(Duration::from_secs(constants::MAX_NEWER_THAN as u64))
            ),
        }
    }
}

pub fn validate_relative_duration(
    newer_than: Option<i64>,
    older_than: Option<i64>,
) -> Result<(), RelativeDurationErr> {
    if let Some(newer_than) = newer_than {
        if let Some(older_than) = older_than {
            if older_than >= newer_than && older_than != 0 && newer_than != 0 {
                return Err(RelativeDurationErr::OlderThanGreaterThanNewerThan);
            }
        }
    }
    if let Some(older_than) = older_than {
        if older_than < 0 {
            return Err(RelativeDurationErr::OlderThanNegative);
        }
        if older_than > constants::MAX_OLDER_THAN {
            return Err(RelativeDurationErr::OlderThanTooLarge);
        }
    }
    if let Some(newer_than) = newer_than {
        if newer_than < 0 {
            return Err(RelativeDurationErr::NewerThanNegative);
        }
        if newer_than > constants::MAX_NEWER_THAN {
            return Err(RelativeDurationErr::NewerThanTooLarge);
        }
    }

    Ok(())
}
