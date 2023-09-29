use snafu::Backtrace;

use crate::get_status;

pub type StarboardResult<T> = Result<T, StarboardError>;

#[derive(Debug, snafu::Snafu)]
pub enum StarboardError {
    #[snafu(context(false))]
    Sqlx {
        source: sqlx::Error,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    Serde {
        source: serde_json::Error,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    TwilightHttp {
        source: twilight_http::Error,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    MessageValidationError {
        source: twilight_validate::message::MessageValidationError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    ValidationError {
        source: twilight_validate::request::ValidationError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    DeserializeBodyError {
        source: twilight_http::response::DeserializeBodyError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    Reqwest {
        source: reqwest::Error,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    InteractionParseError {
        source: twilight_interactions::error::ParseError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    ReceiveMessageError {
        source: twilight_gateway::error::ReceiveMessageError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    SendError {
        source: twilight_gateway::error::SendError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    JoinError {
        source: tokio::task::JoinError,
        backtrace: Backtrace,
    },
    #[snafu(context(false))]
    RegexError {
        source: regex::Error,
        backtrace: Backtrace,
    },
}

impl StarboardError {
    pub fn http_status(&self) -> Option<u16> {
        match &self {
            Self::TwilightHttp { source, .. } => get_status(source),
            _ => None,
        }
    }
}
