use serenity::all::{CommandDataOptionValue, CommandOptionType, UserId};

use super::{CommandOption, ResolveRequired, Unvalidated, autocomplete::NotAutocompletable};
use crate::interaction::error::Result;

impl CommandOption for UserId {
    const KIND: CommandOptionType = CommandOptionType::User;
}

impl Unvalidated for UserId {
    type Unvalidated = NotAutocompletable;
}

impl ResolveRequired for UserId {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_user_id().ok_or(Self::unexpected_type(data).into())
    }
}
