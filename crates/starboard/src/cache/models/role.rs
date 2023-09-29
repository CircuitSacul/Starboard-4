use twilight_model::guild::Role;

pub struct CachedRole {
    pub position: i64,
    pub name: String,
}

impl From<&Role> for CachedRole {
    fn from(value: &Role) -> Self {
        Self {
            position: value.position,
            name: value.name.to_owned(),
        }
    }
}
