use std::fmt;

use serenity::{
    builder::CreateApplicationCommandOption,
    model::{
        application::{
            command::CommandOptionType,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        },
        channel::{Attachment, PartialChannel},
        guild::{PartialMember, Role},
        user::User,
    },
};

use crate::AnyError;

#[derive(Debug)]
pub enum ResolveError {
    Missing,
    Unresolvable,
    InvalidType,
    IntegerRangeError,
    NumberRangeError,
    StringLengthError,
    NoPartialMemberData,
    Other(AnyError),
}

pub type ResolveResult<T> = Result<T, ResolveError>;

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::Missing => write!(f, "parameter not specified"),
            ResolveError::Unresolvable => write!(f, "paremeter is unresolvable"),
            ResolveError::InvalidType => write!(f, "parameter has invalid type"),
            ResolveError::IntegerRangeError => write!(f, "integer out of range"),
            ResolveError::NumberRangeError => write!(f, "number out of range"),
            ResolveError::StringLengthError => write!(f, "invalid string length"),
            ResolveError::NoPartialMemberData => write!(f, "no partial member data available"),
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

    fn describe(_option: &mut CreateApplicationCommandOption) {}

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self>;
}

impl<T: Resolve> Resolve for Option<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = false;

    fn describe(option: &mut CreateApplicationCommandOption) {
        T::describe(option);
    }

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
    ($($command_option_type:ident => $t:ty),* $(,)?) => { $(
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
    )* };
}

impl_resolve! {
    String => String,
    Integer => i64,
    Boolean => bool,
    Channel => PartialChannel,
    Role => Role,
    Number => f64,
    Attachment => Attachment,
}

macro_rules! impl_resolve_for_integer {
    ($($t:ty),* $(,)?) => { $(
        impl Resolve for $t {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            fn describe(option: &mut CreateApplicationCommandOption) {
                i64::try_from(<$t>::MIN).ok().map(|min| option.min_int_value(min));
                i64::try_from(<$t>::MAX).ok().map(|max| option.max_int_value(max));
            }

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
        }
    )* };
}

impl_resolve_for_integer!(i8, i16, i32, i128, isize, u8, u16, u32, u64, u128, usize);

impl Resolve for f32 {
    const KIND: CommandOptionType = CommandOptionType::Number;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
            CommandDataOptionValue::Number(value) => Ok(*value as _),
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

impl Resolve for PartialMember {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve<'a>(
        name: &str,
        options: impl Iterator<Item = &'a CommandDataOption>,
    ) -> ResolveResult<Self> {
        find_and_resolve_option::<Self>(name, options).and_then(|value| match value {
            CommandDataOptionValue::User(_, Some(value)) => Ok(value.clone()),
            CommandDataOptionValue::User(_, None) => Err(ResolveError::NoPartialMemberData),
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

macro_rules! impl_resolve_for_bounded_integer {
    ($( $t:ty => $b:ident ),* $(,)?) => { $(
        impl<const MIN: $t, const MAX: $t> Resolve for ::bounded_integer::$b<MIN, MAX> {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            fn describe(option: &mut CreateApplicationCommandOption) {
                i64::try_from(MIN).ok().map(|min| option.min_int_value(min));
                i64::try_from(MAX).ok().map(|max| option.max_int_value(max));
            }

            fn resolve<'a>(
                name: &str,
                options: impl Iterator<Item = &'a CommandDataOption>,
            ) -> ResolveResult<Self> {
                find_and_resolve_option::<i64>(name, options).and_then(|value| match *value {
                    CommandDataOptionValue::Integer(value) => Self::new(
                        <$t>::try_from(value).map_err(|error| ResolveError::Other(error.into()))?,
                    )
                    .ok_or(ResolveError::IntegerRangeError),
                    _ => Err(ResolveError::InvalidType),
                })
            }
        }
    )* };
}

impl_resolve_for_bounded_integer! {
    i8 => BoundedI8,
    i16 => BoundedI16,
    i32 => BoundedI32,
    i64 => BoundedI64,
    i128 => BoundedI128,
    isize => BoundedIsize,
    u8 => BoundedU8,
    u16 => BoundedU16,
    u32 => BoundedU32,
    u64 => BoundedU64,
    u128 => BoundedU128,
    usize => BoundedUsize,
}

#[macro_export]
macro_rules! bounded_number {
    (@make($v:vis $name:ident: $min:expr, $max:expr $(,)?)) => {
        #[derive(
            ::std::clone::Clone,
            ::std::marker::Copy,
            ::std::fmt::Debug,
            // No default derive or manual implementation, as 0.0 might lie outside the valid range.
            ::std::cmp::PartialEq,
            ::std::cmp::PartialOrd,
        )]
        $v struct $name(::std::primitive::f64);

        impl $crate::resolve::Resolve for $name {
            const KIND: $crate::serenity::model::application::command::CommandOptionType =
                <::std::primitive::f64 as $crate::resolve::Resolve>::KIND;

            fn describe(option: &mut $crate::serenity::builder::CreateApplicationCommandOption) {
                $min.map(|min| option.min_number_value(min));
                $max.map(|max| option.max_number_value(max));
            }

            fn resolve<'a>(
                name: &::std::primitive::str,
                options: impl Iterator<Item = &'a $crate::serenity::model::application::interaction::application_command::CommandDataOption>,
            ) -> $crate::resolve::ResolveResult<Self> {
                <::std::primitive::f64 as $crate::resolve::Resolve>::resolve(name, options).and_then(Self::try_from)
            }
        }

        impl ::std::convert::From<$name> for ::std::primitive::f64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::std::convert::TryFrom<::std::primitive::f64> for $name {
            type Error = $crate::resolve::ResolveError;

            fn try_from(value: ::std::primitive::f64) -> ::std::result::Result<Self, Self::Error> {
                let min_ok = $min.map_or(true, |min| value >= min);
                let max_ok = $max.map_or(true, |max| value <= max);
                if min_ok && max_ok {
                    ::std::result::Result::Ok(Self(value))
                } else {
                    ::std::result::Result::Err($crate::resolve::ResolveError::NumberRangeError)?
                }
            }
        }
    };

    // Avoid unused braces/parens warning.
    (@inner(($value:expr))) => { $value };
    (@inner({$value:expr})) => { $value };
    (@inner($value:expr)) => { $value };

    ($v:vis $name:ident: $min:tt..) => {
        $crate::bounded_number!(@make($v $name:
            ::std::option::Option::Some($crate::bounded_number!(@inner($min))),
            ::std::option::Option::None,
        ));
    };
    ($v:vis $name:ident: ..=$max:tt) => {
        $crate::bounded_number!(@make($v $name:
            ::std::option::Option::None,
            ::std::option::Option::Some($crate::bounded_number!(@inner($max))),
        ));
    };
    ($v:vis $name:ident: $min:tt..=$max:tt) => {
        $crate::bounded_number!(@make($v $name:
            ::std::option::Option::Some($crate::bounded_number!(@inner($min))),
            ::std::option::Option::Some($crate::bounded_number!(@inner($max))),
        ));
    };
}

#[macro_export]
macro_rules! bounded_string {
    (@make($v:vis $name:ident: $min:expr, $max:expr $(,)?)) => {
        #[derive(
            ::std::clone::Clone,
            ::std::fmt::Debug,
            // No default derive or manual implementation, as the empty string might be invalid.
            ::std::cmp::Eq,
            ::std::hash::Hash,
            ::std::cmp::Ord,
            ::std::cmp::PartialEq,
            ::std::cmp::PartialOrd,
        )]
        $v struct $name(::std::string::String);

        impl $crate::resolve::Resolve for $name {
            const KIND: $crate::serenity::model::application::command::CommandOptionType =
                <::std::string::String as $crate::resolve::Resolve>::KIND;

            fn describe(option: &mut $crate::serenity::builder::CreateApplicationCommandOption) {
                $min.map(|min| option.min_length(min));
                $max.map(|max| option.max_length(max));
            }

            fn resolve<'a>(
                name: &::std::primitive::str,
                options: impl Iterator<Item = &'a $crate::serenity::model::application::interaction::application_command::CommandDataOption>,
            ) -> $crate::resolve::ResolveResult<Self> {
                <::std::string::String as $crate::resolve::Resolve>::resolve(name, options).and_then(Self::try_from)
            }
        }

        impl ::std::convert::From<$name> for ::std::string::String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::std::convert::TryFrom<::std::string::String> for $name {
            type Error = $crate::resolve::ResolveError;

            fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, Self::Error> {
                let min_ok = $min.map_or(true, |min: u16| value.len() >= ::std::primitive::usize::from(min));
                let max_ok = $max.map_or(true, |max: u16| value.len() <= ::std::primitive::usize::from(max));
                if min_ok && max_ok {
                    ::std::result::Result::Ok(Self(value))
                } else {
                    ::std::result::Result::Err($crate::resolve::ResolveError::StringLengthError)?
                }
            }
        }
    };

    // Avoid unused braces/parens warning.
    (@inner(($value:expr))) => { $value };
    (@inner({$value:expr})) => { $value };
    (@inner($value:expr)) => { $value };

    ($v:vis $name:ident: $min:tt..) => {
        $crate::bounded_string!(@make($v $name:
            ::std::option::Option::Some($crate::bounded_string!(@inner($min))),
            ::std::option::Option::None,
        ));
    };
    ($v:vis $name:ident: ..=$max:tt) => {
        $crate::bounded_string!(@make($v $name:
            ::std::option::Option::None,
            ::std::option::Option::Some($crate::bounded_string!(@inner($max))),
        ));
    };
    ($v:vis $name:ident: $min:tt..=$max:tt) => {
        $crate::bounded_string!(@make($v $name:
            ::std::option::Option::Some($crate::bounded_string!(@inner($min))),
            ::std::option::Option::Some($crate::bounded_string!(@inner($max))),
        ));
    };
}
