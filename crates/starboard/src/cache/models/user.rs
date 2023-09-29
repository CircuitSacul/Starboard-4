use twilight_model::{user::User, util::ImageHash};

pub struct CachedUser {
    pub is_bot: bool,
    pub name: String,
    pub avatar_hash: Option<ImageHash>,
}

impl From<&User> for CachedUser {
    fn from(user: &User) -> Self {
        Self {
            is_bot: user.bot,
            name: user.name.clone(),
            avatar_hash: user.avatar,
        }
    }
}

impl From<User> for CachedUser {
    fn from(user: User) -> Self {
        Self {
            is_bot: user.bot,
            name: user.name,
            avatar_hash: user.avatar,
        }
    }
}
