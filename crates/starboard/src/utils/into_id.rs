use twilight_model::id::Id;

pub trait IntoId<T> {
    fn into_id(self) -> Id<T>;
}

impl<T> IntoId<T> for i64 {
    fn into_id(self) -> Id<T> {
        Id::new(self as u64)
    }
}

impl<T> IntoId<T> for u64 {
    fn into_id(self) -> Id<T> {
        Id::new(self)
    }
}
