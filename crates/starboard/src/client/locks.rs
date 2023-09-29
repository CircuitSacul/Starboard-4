use twilight_model::id::{marker::MessageMarker, Id};

use crate::utils::dashset_lock::DashSetLock;

#[derive(Default)]
pub struct Locks {
    pub post_update_lock: DashSetLock<i64>,
    pub guild_pr_update: DashSetLock<i64>,
    pub vote_recount: DashSetLock<Id<MessageMarker>>,
}

impl Locks {
    pub fn new() -> Self {
        Self::default()
    }
}
