use twilight_interactions::command::{CommandOption, CreateOption};

#[derive(CreateOption, CommandOption)]
pub enum Tribool {
    #[option(name = "Default", value = "None")]
    Default,
    #[option(name = "True", value = "True")]
    True,
    #[option(name = "False", value = "False")]
    False,
}

impl Tribool {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Default => None,
            Self::True => Some(true),
            Self::False => Some(false),
        }
    }
}
