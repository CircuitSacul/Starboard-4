use twilight_model::guild::Permissions;

pub fn manage_roles() -> Permissions {
    Permissions::MANAGE_ROLES
}

pub fn manage_channels() -> Permissions {
    Permissions::MANAGE_CHANNELS
}

pub fn manage_roles_channels() -> Permissions {
    manage_channels() | manage_roles()
}

pub fn manage_messages() -> Permissions {
    Permissions::MANAGE_MESSAGES
}
