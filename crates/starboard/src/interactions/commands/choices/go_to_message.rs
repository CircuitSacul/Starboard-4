use twilight_interactions::command::{CommandOption, CreateOption};

#[derive(CommandOption, CreateOption)]
pub enum GoToMessage {
    #[option(name = "None", value = 0)]
    None,
    #[option(name = "Link", value = 1)]
    Link,
    #[option(name = "Button", value = 2)]
    Button,
    #[option(name = "Mention", value = 3)]
    Mention,
}
