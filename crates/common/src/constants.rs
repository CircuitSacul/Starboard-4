use std::time::Duration;

pub const ZWS: &str = "\u{200B}";

pub const EMBED_DARK_BG: u32 = 0x2B2D31;

pub const BOT_COLOR: u32 = 0xFFE19C;

pub const YEAR_SECONDS: i64 = 31_557_600;
pub const MONTH_SECONDS: i64 = 2_630_016;
pub const MONTH_DAYS: u64 = 31;

pub const CREDITS_PER_MONTH: u64 = 3;

// Links
pub const INVITE_URL: &str = "https://discord.com/api/oauth2/authorize?client_id=700796664276844612&permissions=805661760&scope=bot%20applications.commands";
pub const SUPPORT_URL: &str = "https://discord.gg/3gK8mSA";
pub const SOURCE_URL: &str = "https://github.com/CircuitSacul/Starboard-4";
pub const PATREON_URL: &str = "https://patreon.com/CircuitSacul";
pub const VOTE_URL: &str = "https://top.gg/bot/700796664276844612/vote";
pub const REVIEW_URL: &str = "https://top.gg/bot/700796664276844612#reviews";

// Docs
pub const DOCS_URL: &str = "https://docs.starboard.best";
pub const DOCS_FILTERS: &str = "https://docs.starboard.best/docs/filters";

// Tasks
pub const UPDATE_PRS_DELAY: Duration = Duration::from_secs(60 * 60);
pub const CHECK_EXPIRED_PREMIUM: Duration = Duration::from_secs(60 * 60);
pub const UPDATE_PATREON_DELAY: Duration = Duration::from_secs(60);
pub const UPDATE_SUPPORTER_ROLES_DELAY: Duration = Duration::from_secs(60);

// Cache size
pub const MAX_MESSAGES: u64 = 50_000;
pub const MESSAGES_TTI: Duration = Duration::from_secs(60 * 60);
pub const MAX_USERS: u64 = 50_000;
pub const USERS_TTI: Duration = Duration::from_secs(60 * 60);
pub const MAX_MEMBERS: u64 = 50_000;
pub const MEMBERS_TTI: Duration = Duration::from_secs(60 * 60);

pub const MAX_STORED_RESPONSES: u64 = 100;
pub const STORED_RESPONSES_TTI: Duration = Duration::from_secs(60 * 5);
pub const MAX_STORED_AUTO_DELETES: usize = 1_000;

// Cooldowns
pub const AUTOSTAR_COOLDOWN: (u64, Duration) = (5, Duration::from_secs(20));
pub const PREM_AUTOSTAR_COOLDOWN: (u64, Duration) = (100, Duration::from_secs(10));
pub const MESSAGE_EDIT: (u64, Duration) = (2, Duration::from_secs(10));
pub const XP_REFRESH: (u64, Duration) = (1, Duration::from_secs(60 * 10));
pub const VOTE_RECOUNT: (u64, Duration) = (5, Duration::from_secs(30));

// Common Validation
pub const MAX_NAME_LENGTH: u32 = 32;
pub const MIN_NAME_LENGTH: u32 = 3;
pub const MAX_REGEX_LENGTH: u32 = 1_000;
pub const MAX_NEWER_THAN: i64 = YEAR_SECONDS * 50;
pub const MAX_OLDER_THAN: i64 = YEAR_SECONDS * 50;

// AutoStar Validation
pub const MAX_MAX_CHARS: i16 = 5_000;
pub const MAX_MIN_CHARS: i16 = 5_000;

pub const MAX_ASC_EMOJIS: usize = 3;
pub const MAX_PREM_ASC_EMOJIS: usize = 20;
pub const MAX_AUTOSTAR: i64 = 3;
pub const MAX_PREM_AUTOSTAR: i64 = 50;

// Starboard Validation
pub const MIN_REQUIRED: i16 = 1;
pub const MAX_REQUIRED: i16 = 10_000;
pub const MIN_REQUIRED_REMOVE: i16 = -10_000;
pub const MAX_REQUIRED_REMOVE: i16 = 9_999;
pub const MIN_XP_MULTIPLIER: f32 = -10.0;
pub const MAX_XP_MULTIPLIER: f32 = 10.0;
pub const MAX_COOLDOWN_CAPACITY: i16 = 3600;
// WARNING: if you make this greater than 1 hour, you have
//          to change the cycle period used by the cooldown
//          struct.
pub const MAX_COOLDOWN_PERIOD: i16 = 3600;

pub const MAX_VOTE_EMOJIS: usize = 3;
pub const MAX_PREM_VOTE_EMOJIS: usize = 20;
pub const MAX_STARBOARDS: i64 = 3;
pub const MAX_PREM_STARBOARDS: i64 = 20;

// Override Validation
pub const MAX_CHANNELS_PER_OVERRIDE: usize = 100;
pub const MAX_OVERRIDES_PER_STARBOARD: i64 = 10;

// Filter Validation
pub const MAX_FILTER_GROUPS: usize = 50;
pub const MAX_FILTERS_PER_GROUP: usize = 9;

pub const MAX_FILTER_ROLES: usize = 25;
pub const MAX_FILTER_CHANNELS: usize = 25;
pub const MAX_ATTACHMENTS: i64 = 10;
pub const MAX_LENGTH: i64 = 5_000;

// Exclusive Group Validation
pub const MAX_EXCLUSIVE_GROUPS: i64 = 10;

// PermRole Validation
pub const MAX_PERMROLES: i64 = 50;

// XP-based Award Role Validation
pub const MAX_XPROLES: i64 = 50;

// Position-based Award Role Validation
pub const MAX_POSROLES: i64 = 50;
