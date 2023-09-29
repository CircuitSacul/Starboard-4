use std::{sync::Arc, time::Duration};

use floodgate::{DynamicMapping, FixedMapping};
use tokio::time::sleep;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

use common::constants;

use super::bot::StarboardBot;

pub struct Cooldowns {
    cycle_period: Duration,
    // restricts per-channel
    pub autostar_send: FixedMapping<Id<GuildMarker>>,
    pub prem_autostar_send: FixedMapping<Id<GuildMarker>>,
    pub starboard_custom_cooldown: DynamicMapping<(Id<UserMarker>, i32)>,
    pub message_edit: FixedMapping<Id<ChannelMarker>>,
    pub xp_refresh: FixedMapping<(Id<UserMarker>, Id<GuildMarker>)>,
    pub vote_recount: FixedMapping<Id<GuildMarker>>,
}

impl Cooldowns {
    pub fn new() -> Self {
        let cycle_period = Duration::from_secs(3600);

        let autostar_send = FixedMapping::new(
            constants::AUTOSTAR_COOLDOWN.0,
            constants::AUTOSTAR_COOLDOWN.1,
        );
        let prem_autostar_send = FixedMapping::new(
            constants::PREM_AUTOSTAR_COOLDOWN.0,
            constants::PREM_AUTOSTAR_COOLDOWN.1,
        );
        let starboard_custom_cooldown = DynamicMapping::new(cycle_period);
        let message_edit = FixedMapping::new(constants::MESSAGE_EDIT.0, constants::MESSAGE_EDIT.1);
        let xp_refresh = FixedMapping::new(constants::XP_REFRESH.0, constants::XP_REFRESH.1);
        let vote_recount = FixedMapping::new(constants::VOTE_RECOUNT.0, constants::VOTE_RECOUNT.1);

        Self {
            cycle_period,
            autostar_send,
            prem_autostar_send,
            starboard_custom_cooldown,
            message_edit,
            xp_refresh,
            vote_recount,
        }
    }

    pub fn start(bot: Arc<StarboardBot>) {
        tokio::spawn(async move {
            let cooldown = &bot.cooldowns;
            loop {
                sleep(cooldown.cycle_period).await;

                cooldown.autostar_send.cycle();
                cooldown.prem_autostar_send.cycle();
                cooldown.starboard_custom_cooldown.cycle();
                cooldown.message_edit.cycle();
                cooldown.xp_refresh.cycle();
                cooldown.vote_recount.cycle();
            }
        });
    }
}

impl Default for Cooldowns {
    fn default() -> Self {
        Self::new()
    }
}
