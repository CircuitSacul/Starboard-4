use thiserror::Error;

pub type StarboardResult<T> = Result<T, StarboardError>;

#[derive(Error, Debug)]
pub enum StarboardError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    TwilightHttp(#[from] twilight_http::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    InteractionParseError(#[from] twilight_interactions::error::ParseError),
    #[error(transparent)]
    ClusterCommandError(#[from] twilight_gateway::cluster::ClusterCommandError),
    #[error(transparent)]
    ClusterStartoError(#[from] twilight_gateway::cluster::ClusterStartError),
}
