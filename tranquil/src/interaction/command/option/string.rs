use std::{mem::take, ops::Deref};

use serenity::all::{CommandDataOptionValue, CommandOptionType, CreateCommandOption};

use super::{CommandOption, ResolveRequired, Unvalidated};
use crate::interaction::error::{DiscordError, Result};

impl CommandOption for String {
    const KIND: CommandOptionType = CommandOptionType::String;
}

impl Unvalidated for String {
    type Unvalidated = Self;
}

impl ResolveRequired for String {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        if let CommandDataOptionValue::String(data) = data {
            Ok(take(data))
        } else {
            Err(Self::unexpected_type(data).into())
        }
    }
}

pub struct LenString<const MIN_LEN: u16, const MAX_LEN: u16>(String);

impl<const MIN_LEN: u16, const MAX_LEN: u16> Deref for LenString<MIN_LEN, MAX_LEN> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> CommandOption for LenString<MIN_LEN, MAX_LEN> {
    const KIND: CommandOptionType = CommandOptionType::String;

    fn add_validation(create_command_option: CreateCommandOption) -> CreateCommandOption {
        create_command_option
            .min_length(MIN_LEN)
            .max_length(MAX_LEN)
    }
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> Unvalidated for LenString<MIN_LEN, MAX_LEN> {
    type Unvalidated = String;
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> ResolveRequired for LenString<MIN_LEN, MAX_LEN> {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        let string = String::resolve(data)?;
        let got = string.len();
        Self::new(string).ok_or(
            DiscordError::StringLengthOutOfBounds {
                min: MIN_LEN,
                max: MAX_LEN,
                got,
            }
            .into(),
        )
    }
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> LenString<MIN_LEN, MAX_LEN> {
    pub fn new(string: String) -> Option<Self> {
        if (MIN_LEN as usize..=MAX_LEN as usize).contains(&string.len()) {
            Some(Self(string))
        } else {
            None
        }
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}
