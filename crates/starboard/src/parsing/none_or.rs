use std::str::FromStr;

pub fn none_or<T: FromStr>(input: &str) -> Result<Option<T>, T::Err> {
    if input == "none" {
        return Ok(None);
    }

    input.parse().map(Some)
}
