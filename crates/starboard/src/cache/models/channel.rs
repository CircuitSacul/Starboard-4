use twilight_model::{
    channel::{Channel, ChannelType},
    id::{marker::ChannelMarker, Id},
};

pub struct CachedChannel {
    pub is_nsfw: Option<bool>,
    pub parent_id: Option<Id<ChannelMarker>>,
    pub kind: ChannelType,
}

impl CachedChannel {
    pub fn from_channel(original: Option<&CachedChannel>, new: &Channel) -> Self {
        if let Some(original) = original {
            Self {
                is_nsfw: new.nsfw.or(original.is_nsfw),
                parent_id: new.parent_id,
                kind: new.kind,
            }
        } else {
            Self {
                is_nsfw: new.nsfw,
                parent_id: new.parent_id,
                kind: new.kind,
            }
        }
    }
}
