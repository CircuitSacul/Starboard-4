use std::time::Duration;

use twilight_util::snowflake::Snowflake;

pub trait SnowflakeAge {
    /// Snowflake age in seconds
    fn age(&self) -> Duration;
}

impl<T: Snowflake> SnowflakeAge for T {
    fn age(&self) -> Duration {
        let age_millis = chrono::Utc::now().timestamp_millis() - self.timestamp();

        Duration::from_millis(age_millis as u64)
    }
}
