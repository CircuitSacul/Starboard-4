use twilight_http::error::ErrorType;

pub fn get_status(error: &twilight_http::Error) -> Option<u16> {
    if let ErrorType::Response {
        body: _,
        error: _,
        status,
    } = error.kind()
    {
        Some(status.get())
    } else {
        None
    }
}
