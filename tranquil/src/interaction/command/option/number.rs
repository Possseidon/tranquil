use serenity::all::{CommandDataOptionValue, CommandOptionType};

use super::{CommandOption, ResolveRequired, Unvalidated};
use crate::interaction::error::Result;

impl CommandOption for f64 {
    const KIND: CommandOptionType = CommandOptionType::Number;
}

impl Unvalidated for f64 {
    type Unvalidated = Self;
}

impl ResolveRequired for f64 {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_f64().ok_or(Self::unexpected_type(data).into())
    }
}

// TODO: BoundedNumber macro that creates a new type with appropriate bounds
