use serenity::all::{CommandDataOptionValue, CommandOptionType};

use super::{CommandOption, ResolveRequired, Unvalidated, autocomplete::NotAutocompletable};
use crate::interaction::error::Result;

impl CommandOption for bool {
    const KIND: CommandOptionType = CommandOptionType::Boolean;
}

impl Unvalidated for bool {
    type Unvalidated = NotAutocompletable;
}

impl ResolveRequired for bool {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_bool().ok_or(Self::unexpected_type(data).into())
    }
}
