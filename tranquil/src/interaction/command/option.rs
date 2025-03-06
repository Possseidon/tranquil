pub mod attachment;
pub mod autocomplete;
pub mod boolean;
pub mod channel;
pub mod integer;
pub mod mentionable;
pub mod number;
pub mod role;
pub mod string;
pub mod user;

use serenity::all::{
    CommandDataOptionValue, CommandDataResolved, CommandOptionType, CreateCommandOption,
};

use crate::interaction::error::{DiscordError, Result};

pub trait CommandOption {
    const KIND: CommandOptionType;

    /// Returns a [`CreateCommandOption`] with all relevant client-side validation.
    fn create(name: String, description: String) -> CreateCommandOption {
        Self::add_validation(CreateCommandOption::new(Self::KIND, name, description).required(true))
    }

    /// Adds validation to the given [`CreateCommandOption`].
    fn add_validation(create_command_option: CreateCommandOption) -> CreateCommandOption {
        create_command_option
    }

    fn unexpected_type(data: &CommandDataOptionValue) -> DiscordError {
        DiscordError::UnexpectedOptionType {
            expected: Self::KIND,
            got: data.kind(),
        }
    }
}

impl<T: CommandOption> CommandOption for Option<T> {
    const KIND: CommandOptionType = T::KIND;

    fn add_validation(create_command_option: CreateCommandOption) -> CreateCommandOption {
        T::add_validation(create_command_option).required(false)
    }
}

pub trait Unvalidated {
    /// Set to [`NotAutocompletable`] for anything that isn't a string, integer or number.
    type Unvalidated: ResolveRequired;
}

impl<T: ResolveRequired> Unvalidated for Option<T> {
    type Unvalidated = T;
}

pub trait Resolve: Sized {
    fn resolve(data: Option<&mut CommandDataOptionValue>) -> Result<Self>;
}

impl<T: ResolveRequired> Resolve for T {
    fn resolve(data: Option<&mut CommandDataOptionValue>) -> Result<Self> {
        if let Some(data) = data {
            T::resolve(data)
        } else {
            Err(DiscordError::MissingRequiredOption.into())
        }
    }
}

impl<T: ResolveRequired> Resolve for Option<T> {
    fn resolve(data: Option<&mut CommandDataOptionValue>) -> Result<Self> {
        Ok(if let Some(data) = data {
            Some(T::resolve(data)?)
        } else {
            None
        })
    }
}

pub trait ResolveRequired: Sized {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self>;
}
