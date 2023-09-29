//! Provides access to a messages "system" content,
//! which includes the normal content in most cases,
//! but tries to replicate what a discord client sees
//! for join messages, boost messages, and other
//! "system" messages.

use twilight_model::channel::{message::MessageType, Message};

use crate::concat_format;

pub trait SystemContent {
    fn system_content(&self) -> String;
}

impl SystemContent for Message {
    fn system_content(&self) -> String {
        match self.kind {
            MessageType::Regular
            | MessageType::Unknown(_)
            | MessageType::Reply
            | MessageType::ChatInputCommand
            | MessageType::ContextMenuCommand
            | MessageType::AutoModerationAction => self.content.to_string(),
            MessageType::Call => unreachable!(),
            // doesn't handle group DMs
            MessageType::RecipientAdd => format!(
                "**{}** added **{}** to the thread.",
                self.author.name, self.mentions[0].name
            ),
            // doesn't handle group DMs
            MessageType::RecipientRemove => format!(
                "**{}** removed **{}** from the thread.",
                self.author.name, self.mentions[0].name
            ),
            // TODO differentiate between Forums and Threads
            MessageType::ChannelNameChange => format!(
                "**{}** changed the channel name: **{}**",
                self.author.name, self.content
            ),
            MessageType::ChannelIconChange => {
                format!("**{}** changed the channel icon.", self.author.name)
            }
            MessageType::UserJoin => {
                let created_at = (self.timestamp.as_micros() / 1000) as u64;
                match created_at % 13 {
                    0 => format!("**{}** joined the party.", self.author.name),
                    1 => format!("**{}** is here.", self.author.name),
                    2 => format!(
                        "Welcome, **{}**. We hope you brought pizza.",
                        self.author.name
                    ),
                    3 => format!("A wild **{}** appeared.", self.author.name),
                    4 => format!("**{}** just landed.", self.author.name),
                    5 => format!("**{}** just slid into the server.", self.author.name),
                    6 => format!("**{}** just showed up!", self.author.name),
                    7 => format!("Welcome **{}**. Say hi!", self.author.name),
                    8 => format!("**{}** just hopped into the server.", self.author.name),
                    9 => format!("Everyone welcome **{}**!", self.author.name),
                    10 => format!("Glad you're here, **{}**.", self.author.name),
                    11 => format!("Good to see you, **{}**.", self.author.name),
                    12 => format!("Yay you made it, **{}**!", self.author.name),
                    _ => unreachable!(),
                }
            }
            MessageType::GuildBoost => {
                if self.content.is_empty() {
                    format!("**{}** just boosted the server!", self.author.name)
                } else {
                    format!(
                        "**{}** just boosted the server **{}** times!",
                        self.author.name, self.content
                    )
                }
            }
            // TODO get actual guild name
            MessageType::GuildBoostTier1 => {
                if self.content.is_empty() {
                    format!(
                        "**{}** just boosted the server! This server has achieved **Level 1**!",
                        self.author.name
                    )
                } else {
                    concat_format!(
                        "**{}** just boosted the server **{}** " <- self.author.name, self.content;
                        "times! This server has achieved **Level 1**!";
                    )
                }
            }
            MessageType::GuildBoostTier2 => {
                if self.content.is_empty() {
                    format!(
                        "**{}** just boosted the server! This server has achieved **Level 2**!",
                        self.author.name
                    )
                } else {
                    concat_format!(
                        "{} just boosted the server **{}** " <- self.author.name, self.content;
                        "times! This server has achieved **Level 2**!";
                    )
                }
            }
            MessageType::GuildBoostTier3 => {
                if self.content.is_empty() {
                    format!(
                        "**{}** just boosted the server! This server has achieved **Level 3**!",
                        self.author.name
                    )
                } else {
                    concat_format!(
                        "**{}** just boosted the server **{}** " <- self.author.name, self.content;
                        "times! This server has achieved **Level 3**!";
                    )
                }
            }
            MessageType::ChannelFollowAdd => concat_format!(
                "**{}** has added **{}** " <- self.author.name, self.content;
                "to this channel. Its most important updates will show up here.";
            ),
            MessageType::GuildDiscoveryDisqualified => concat!(
                "This server has been removed from Server Discovery because it no longer passes ",
                "all the requirements. Check Server Settings for more details.",
            )
            .to_string(),
            MessageType::GuildDiscoveryRequalified => concat!(
                "This server is eligible for Server Discovery again and has been automatically ",
                "relisted!",
            )
            .to_string(),
            MessageType::GuildDiscoveryGracePeriodInitialWarning => concat!(
                "This server has failed Discovery activity requirements for 1 week. If this ",
                "server fails for 4 weeks in a row, it will be automatically removed from ",
                "Discovery.",
            )
            .to_string(),
            MessageType::GuildDiscoveryGracePeriodFinalWarning => concat!(
                "This server has failed Discovery activity requirements for 3 weeks in a row. ",
                "If this server fails for 1 more week, it will be removed from Discovery.",
            )
            .to_string(),
            MessageType::ThreadCreated => format!(
                "**{}** started a thread: **{}**. See all **threads**.",
                self.author.name, self.content
            ),
            MessageType::ThreadStarterMessage => match &self.referenced_message {
                None => "Sorry, we couldn't load the first message in this thread".to_string(),
                Some(msg) => msg.content.clone(),
            },
            MessageType::GuildInviteReminder => concat!(
                "Wondering who to invite?\nStart by inviting anyone who can help you build the ",
                "server!",
            )
            .to_string(),
            MessageType::ChannelMessagePinned => format!(
                "**{}** pinned **a message** to this channel. See all **pinned messages**.",
                self.author.name,
            ),
            MessageType::StageStart => format!("{} started **{}**", self.author.name, self.content),
            MessageType::StageEnd => format!("{} ended **{}**", self.author.name, self.content),
            MessageType::StageTopic => format!(
                "{} changed the Stage topic: **{}**",
                self.author.name, self.content
            ),
            MessageType::StageSpeaker => format!("{} is now a speaker.", self.author.name),
            unkown => {
                eprintln!("Warning: unkown message type {unkown:?}");
                self.content.clone()
            }
        }
    }
}
