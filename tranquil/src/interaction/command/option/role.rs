use serenity::all::{CommandDataOptionValue, CommandOptionType, RoleId};

use super::{CommandOption, ResolveRequired, Unvalidated, autocomplete::NotAutocompletable};
use crate::interaction::error::Result;

impl CommandOption for RoleId {
    const KIND: CommandOptionType = CommandOptionType::Role;
}

impl Unvalidated for RoleId {
    type Unvalidated = NotAutocompletable;
}

impl ResolveRequired for RoleId {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_role_id().ok_or(Self::unexpected_type(data).into())
    }
}
