use std::fmt;

use serenity::model::{
    application::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    },
    channel::{Attachment, PartialChannel},
    guild::{PartialMember, Role},
    user::User,
};

use crate::AnyError;

#[derive(Debug)]
pub enum ResolveError {
    Missing,
    Unresolvable,
    InvalidType,
    Other(AnyError),
}

type ResolveResult<T> = Result<T, ResolveError>;

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::Missing => write!(f, "parameter not specified"),
            ResolveError::Unresolvable => write!(f, "paremeter is unresolvable"),
            ResolveError::InvalidType => write!(f, "parameter has invalid type"),
            ResolveError::Other(error) => error.fmt(f),
        }
    }
}

fn find_option<'a, T>(
    name: &str,
    mut options: impl Iterator<Item = &'a CommandDataOption>,
) -> ResolveResult<&'a CommandDataOption> {
    options
        .find(|option| option.name == name)
        .ok_or(ResolveError::Missing)
}

fn resolve_option(option: &CommandDataOption) -> ResolveResult<&CommandDataOptionValue> {
    option.resolved.as_ref().ok_or(ResolveError::Unresolvable)
}

fn find_and_resolve_option<'a, T>(
    name: &str,
    options: impl Iterator<Item = &'a CommandDataOption>,
) -> ResolveResult<&'a CommandDataOptionValue> {
    find_option::<T>(name, options).and_then(resolve_option)
}

pub trait Resolve: Sized {
    const KIND: CommandOptionType;
    const REQUIRED: bool = true;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self>;
}

impl<T: Resolve> Resolve for Option<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = false;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        T::resolve(name, options)
            .map(|value| Some(value))
            .or_else(|error| match error {
                ResolveError::Missing => Ok(None),
                error => Err(error),
            })
    }
}

macro_rules! impl_resolve {
    ($command_option_type:ident => $t:ty) => {
        impl Resolve for $t {
            const KIND: CommandOptionType = CommandOptionType::$command_option_type;

            fn resolve<'a>(
                name: &str,
                options: impl Iterator<Item = &'a CommandDataOption>,
            ) -> ResolveResult<Self> {
                find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
                    CommandDataOptionValue::$command_option_type(value) => Ok(value.clone()),
                    _ => Err(ResolveError::InvalidType),
                })
            }
        }
    };
}

impl_resolve!(String => String);
impl_resolve!(Integer => i64);
impl_resolve!(Boolean => bool);
impl_resolve!(Channel => PartialChannel);
impl_resolve!(Role => Role);
impl_resolve!(Number => f64);
impl_resolve!(Attachment => Attachment);

macro_rules! impl_resolve_for_integer {
    ($( $t:ty ),*) => {
        $( impl Resolve for $t {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            fn resolve<'a>(
                name: &str,
                options: impl Iterator<Item = &'a CommandDataOption>,
            ) -> ResolveResult<Self> {
                find_and_resolve_option::<i64>(name, options).and_then(|value| match *value {
                    CommandDataOptionValue::Integer(value) => {
                        <$t>::try_from(value).map_err(|error| ResolveError::Other(error.into()))
                    }
                    _ => Err(ResolveError::InvalidType),
                })
            }
        } )*
    };
}

impl_resolve_for_integer!(i8, i16, i32, i128, u8, u16, u32, u64, u128);

impl Resolve for f32 {
    const KIND: CommandOptionType = CommandOptionType::Number;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
            CommandDataOptionValue::Number(value) => Ok(*value as Self),
            _ => Err(ResolveError::InvalidType),
        })
    }
}

impl Resolve for User {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
            CommandDataOptionValue::User(value, _) => Ok(value.clone()),
            _ => Err(ResolveError::InvalidType),
        })
    }
}

impl Resolve for Option<PartialMember> {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
            CommandDataOptionValue::User(_, value) => Ok(value.clone()),
            _ => Err(ResolveError::InvalidType),
        })
    }
}

#[derive(Debug)]
pub enum Mentionable {
    User(User, Option<PartialMember>),
    Role(Role),
}

impl Resolve for Mentionable {
    const KIND: CommandOptionType = CommandOptionType::Mentionable;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
            CommandDataOptionValue::User(user, partial_member) => {
                Ok(Mentionable::User(user.clone(), partial_member.clone()))
            }
            CommandDataOptionValue::Role(role) => Ok(Mentionable::Role(role.clone())),
            _ => Err(ResolveError::InvalidType),
        })
    }
}
