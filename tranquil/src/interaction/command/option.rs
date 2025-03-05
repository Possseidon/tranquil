use std::{
    mem::take,
    num::{NonZero, NonZeroU16},
};

use anyhow::Result;
use serenity::all::{CommandDataOptionValue, CommandOptionType, CreateCommandOption};
use thiserror::Error;

pub trait CommandOption: Resolve {
    /// Customizes [`CreateCommandOption`] for this type.
    ///
    /// - [`CreateCommandOption::name`], [`CreateCommandOption::description`] and also
    ///   [`CreateCommandOption::kind`] must be set by the caller (before or after).
    /// - [`CreateCommandOption::required`] must be set to `true` initially and will be set to
    ///   `false` for [`Option`] types.
    fn create_command_option(create_command_option: CreateCommandOption) -> CreateCommandOption {
        create_command_option
    }
}

pub trait Resolve: Sized {
    const KIND: CommandOptionType;

    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self>;
}

impl CommandOption for String {}

impl Resolve for String {
    const KIND: CommandOptionType = CommandOptionType::String;

    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        if let CommandDataOptionValue::String(data) = data {
            Ok(take(data))
        } else {
            Err(unexpected_type::<Self>(data))
        }
    }
}

impl CommandOption for i64 {}

impl Resolve for i64 {
    const KIND: CommandOptionType = CommandOptionType::Integer;

    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_i64().ok_or(unexpected_type::<Self>(data))
    }
}

impl CommandOption for f64 {}

impl Resolve for f64 {
    const KIND: CommandOptionType = CommandOptionType::Number;

    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_f64().ok_or(unexpected_type::<Self>(data))
    }
}

impl<T: CommandOption> CommandOption for Option<T> {
    fn create_command_option(create_command_option: CreateCommandOption) -> CreateCommandOption {
        create_command_option.required(false)
    }
}

impl<T: Resolve> Resolve for Option<T> {
    const KIND: CommandOptionType = T::KIND;

    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        todo!("huh? is the thing just omitted entirely?");
    }
}

pub enum Autocompleted<T> {
    Unfocused(T),
    Focused(String),
}

// - if there is any option tagged with `autocomplete`, a copy of that command is created in the
//   Autocomplete type
// - any options that are tagged with `autocomplete` are turned into Autocompleted<T>
// - other options are taken as is
//
// check if min/max for string/number/int is upheld; if not, T needs to turn into some relaxed type
// that is okay with all values (likely including null)
//
// this type is then used by Autcompleted::Unfocused and used as is for non-autocompleted options in
// commands with any autocompleted option

impl<T: Resolve> Resolve for Autocompleted<T> {
    const KIND: CommandOptionType = T::KIND;

    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        if let CommandDataOptionValue::Autocomplete { kind, value } = data {
            if *kind == T::KIND {
                Ok(Self::Focused(take(value)))
            } else {
                Err(unexpected_type::<T>(data))
            }
        } else {
            Ok(Self::Unfocused(T::resolve(data)?))
        }
    }
}

#[derive(Debug, Error)]
pub enum OptionResolveError {
    #[error("cannot resolve option due to wrong type")]
    UnexpectedType {
        expected: CommandOptionType,
        got: CommandOptionType,
    },
}

fn unexpected_type<T: Resolve>(data: &CommandDataOptionValue) -> anyhow::Error {
    OptionResolveError::UnexpectedType {
        expected: T::KIND,
        got: data.kind(),
    }
    .into()
}
