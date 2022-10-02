use std::fmt;

use serenity::model::{
    application::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    },
    prelude::*,
};

use crate::AnyResult;

#[derive(Debug)]
pub enum ResolveError {
    InvalidType,
    Unresolvable,
}

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ResolveError::InvalidType => "parameter has invalid type",
                ResolveError::Unresolvable => "paremeter is unresolvable",
            }
        )
    }
}

pub trait Resolve: Sized {
    const KIND: CommandOptionType;

    fn resolve(option: &CommandDataOption) -> AnyResult<Self>;
}

impl<T: ResolveValue> Resolve for T {
    const KIND: CommandOptionType = T::KIND;

    fn resolve(option: &CommandDataOption) -> AnyResult<Self> {
        match option.resolved.as_ref() {
            Some(value) => Self::resolve_value(value),
            None => Err(ResolveError::Unresolvable)?,
        }
    }
}

pub trait ResolveValue: Sized {
    const KIND: CommandOptionType;

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self>;
}

impl<T: ResolveValue> ResolveValue for Option<T> {
    const KIND: CommandOptionType = T::KIND;

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
        Ok(Some(T::resolve_value(value)?))
    }
}

macro_rules! impl_resolve {
    ($command_option_type:ident => $t:ty) => {
        impl ResolveValue for $t {
            const KIND: CommandOptionType = CommandOptionType::$command_option_type;

            fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
                match value {
                    CommandDataOptionValue::$command_option_type(value) => Ok(value.clone()),
                    _ => Err(ResolveError::InvalidType)?,
                }
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
        $( impl ResolveValue for $t {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
                match value {
                    CommandDataOptionValue::Integer(value) => {
                        Ok(<$t>::try_from(*value)?)
                    }
                    _ => Err(ResolveError::InvalidType)?,
                }
            }
        } )*
    };
}

impl_resolve_for_integer!(i8, i16, i32, i128, u8, u16, u32, u64, u128);

impl ResolveValue for f32 {
    const KIND: CommandOptionType = CommandOptionType::Number;

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
        match value {
            CommandDataOptionValue::Number(value) => Ok(*value as f32),
            _ => Err(ResolveError::InvalidType)?,
        }
    }
}

impl ResolveValue for User {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
        match value {
            CommandDataOptionValue::User(value, _) => Ok(value.clone()),
            _ => Err(ResolveError::InvalidType)?,
        }
    }
}

impl ResolveValue for Option<PartialMember> {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
        match value {
            CommandDataOptionValue::User(_, value) => Ok(value.clone()),
            _ => Err(ResolveError::InvalidType)?,
        }
    }
}

#[derive(Debug)]
pub enum Mentionable {
    User(User, Option<PartialMember>),
    Role(Role),
}

impl ResolveValue for Mentionable {
    const KIND: CommandOptionType = CommandOptionType::Mentionable;

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
        match value {
            CommandDataOptionValue::User(user, partial_member) => {
                Ok(Mentionable::User(user.clone(), partial_member.clone()))
            }
            CommandDataOptionValue::Role(role) => Ok(Mentionable::Role(role.clone())),
            _ => Err(ResolveError::InvalidType)?,
        }
    }
}

pub fn resolve_parameter<R: Resolve>(options: &Vec<CommandDataOption>, name: &str) -> AnyResult<R> {
    options
        .iter()
        .find(|option| option.name == name)
        .map(R::resolve)
        .unwrap_or_else(|| Err(format!("Missing parameter: {name}.").into()))
}
