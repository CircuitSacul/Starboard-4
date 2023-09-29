use twilight_model::id::Id;

pub trait GetI64 {
    fn get_i64(&self) -> i64;
}

impl<T> GetI64 for Id<T> {
    fn get_i64(&self) -> i64 {
        self.get() as i64
    }
}
