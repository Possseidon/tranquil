use serenity::all::{CommandDataOptionValue, CommandOptionType};

use super::{CommandOption, ResolveRequired, Unvalidated};
use crate::interaction::error::Result;

impl CommandOption for i64 {
    const KIND: CommandOptionType = CommandOptionType::Integer;
}

impl Unvalidated for i64 {
    type Unvalidated = Self;
}

impl ResolveRequired for i64 {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_i64().ok_or(Self::unexpected_type(data).into())
    }
}

// TODO: other integer types that limit discord client side validation to its range
// TODO: BoundedInteger types that limit discord client side validation to those bounds
