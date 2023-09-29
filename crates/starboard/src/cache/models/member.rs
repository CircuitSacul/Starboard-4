use std::collections::HashSet;

use twilight_model::{
    gateway::payload::incoming::MemberUpdate,
    guild::Member,
    id::{marker::RoleMarker, Id},
    util::ImageHash,
};

pub struct CachedMember {
    pub nickname: Option<String>,
    pub server_avatar_hash: Option<ImageHash>,
    pub roles: HashSet<Id<RoleMarker>>,
}

impl From<Member> for CachedMember {
    fn from(member: Member) -> Self {
        Self {
            nickname: member.nick,
            server_avatar_hash: member.avatar,
            roles: HashSet::from_iter(member.roles),
        }
    }
}

impl From<&Member> for CachedMember {
    fn from(member: &Member) -> Self {
        Self {
            nickname: member.nick.clone(),
            server_avatar_hash: member.avatar,
            roles: HashSet::from_iter(member.roles.to_owned()),
        }
    }
}

impl From<&MemberUpdate> for CachedMember {
    fn from(member: &MemberUpdate) -> Self {
        Self {
            nickname: member.nick.clone(),
            server_avatar_hash: member.avatar,
            roles: HashSet::from_iter(member.roles.to_owned()),
        }
    }
}
