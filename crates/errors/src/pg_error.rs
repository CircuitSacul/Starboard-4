use std::borrow::Cow;

pub trait PgErrorTraits {
    fn err_code(&self) -> Option<Cow<'_, str>>;
    fn is_duplicate(&self) -> bool;
    fn is_fk_violation(&self) -> bool;
}

impl PgErrorTraits for sqlx::Error {
    fn err_code(&self) -> Option<Cow<'_, str>> {
        match self {
            Self::Database(err) => err.code(),
            _ => None,
        }
    }

    fn is_duplicate(&self) -> bool {
        self.err_code().as_deref() == Some("23505")
    }

    fn is_fk_violation(&self) -> bool {
        self.err_code().as_deref() == Some("23503")
    }
}
