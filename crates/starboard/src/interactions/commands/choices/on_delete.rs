use twilight_interactions::command::{CommandOption, CreateOption};

#[derive(CommandOption, CreateOption)]
pub enum OnDelete {
    #[option(name = "Refresh", value = 0)]
    Refresh,
    #[option(name = "Ignore", value = 1)]
    Ignore,
    #[option(name = "Trash All", value = 2)]
    TrashAll,
    #[option(name = "Freeze All", value = 3)]
    FreezeAll,
}
