use std::fmt;

use serenity::{
    builder::CreateApplicationCommandOption,
    model::{
        application::{
            command::CommandOptionType,
            interaction::application_command::{
                CommandData, CommandDataOption, CommandDataOptionValue,
            },
        },
        channel::{Attachment, PartialChannel},
        guild::{PartialMember, Role},
        user::User,
    },
};

use crate::{l10n::L10n, AnyError};

#[derive(Debug)]
pub enum ResolveError {
    Missing,
    Unresolvable,
    InvalidType,
    IntegerRangeError,
    NumberRangeError,
    StringLengthError,
    NoPartialMemberData,
    InvalidChoice,
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
            ResolveError::InvalidChoice => write!(f, "invalid choice"),
            ResolveError::Other(error) => error.fmt(f),
        }
    }
}

pub fn find_option<'a>(
    name: &str,
    mut options: impl Iterator<Item = &'a CommandDataOption>,
) -> Option<CommandDataOption> {
    // TODO: Avoid clone by passing in an owned iterator.
    options.find_map(|option| (option.name == name).then_some(option.clone()))
}

fn resolve_option(option: Option<CommandDataOption>) -> ResolveResult<CommandDataOptionValue> {
    option.map_or(Err(ResolveError::Missing), |option| {
        option.resolved.ok_or(ResolveError::Unresolvable)
    })
}

pub trait Resolve: Sized {
    const KIND: CommandOptionType;
    const REQUIRED: bool = true;

    fn describe(_option: &mut CreateApplicationCommandOption, _l10n: &L10n) {}

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self>;
}

impl<T: Resolve> Resolve for Option<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = false;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        T::describe(option, l10n);
    }

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
        Ok(match option {
            Some(_) => Some(T::resolve(option)?),
            None => None,
        })
    }
}

macro_rules! impl_resolve {
    ($($command_option_type:ident => $t:ty),* $(,)?) => { $(
        impl Resolve for $t {
            const KIND: CommandOptionType = CommandOptionType::$command_option_type;

            fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
                match resolve_option(option)? {
                    CommandDataOptionValue::$command_option_type(value) => Ok(value),
                    _ => Err(ResolveError::InvalidType),
                }
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

            fn describe(option: &mut CreateApplicationCommandOption, _l10n: &L10n) {
                i64::try_from(<$t>::MIN).ok().map(|min| option.min_int_value(min));
                i64::try_from(<$t>::MAX).ok().map(|max| option.max_int_value(max));
            }

            fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
                match resolve_option(option)? {
                    CommandDataOptionValue::Integer(value) => {
                        <$t>::try_from(value).map_err(|error| ResolveError::Other(error.into()))
                    }
                    _ => Err(ResolveError::InvalidType),
                }
            }
        }
    )* };
}

impl_resolve_for_integer!(i8, i16, i32, i128, isize, u8, u16, u32, u64, u128, usize);

impl Resolve for f32 {
    const KIND: CommandOptionType = CommandOptionType::Number;

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
        match resolve_option(option)? {
            CommandDataOptionValue::Number(value) => Ok(value as _),
            _ => Err(ResolveError::InvalidType),
        }
    }
}

impl Resolve for User {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
        match resolve_option(option)? {
            CommandDataOptionValue::User(value, _) => Ok(value),
            _ => Err(ResolveError::InvalidType),
        }
    }
}

impl Resolve for PartialMember {
    const KIND: CommandOptionType = CommandOptionType::User;

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
        match resolve_option(option)? {
            CommandDataOptionValue::User(_, Some(value)) => Ok(value),
            CommandDataOptionValue::User(_, None) => Err(ResolveError::NoPartialMemberData),
            _ => Err(ResolveError::InvalidType),
        }
    }
}

#[derive(Debug)]
pub enum Mentionable {
    User(User, Option<PartialMember>),
    Role(Role),
}

impl Resolve for Mentionable {
    const KIND: CommandOptionType = CommandOptionType::Mentionable;

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
        match resolve_option(option)? {
            CommandDataOptionValue::User(user, partial_member) => {
                Ok(Mentionable::User(user, partial_member))
            }
            CommandDataOptionValue::Role(role) => Ok(Mentionable::Role(role)),
            _ => Err(ResolveError::InvalidType),
        }
    }
}

macro_rules! impl_resolve_for_bounded_integer {
    ($( $t:ty => $b:ident ),* $(,)?) => { $(
        impl<const MIN: $t, const MAX: $t> Resolve for bounded_integer::$b<MIN, MAX> {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            fn describe(option: &mut CreateApplicationCommandOption, _l10n: &L10n) {
                i64::try_from(MIN).ok().map(|min| option.min_int_value(min));
                i64::try_from(MAX).ok().map(|max| option.max_int_value(max));
            }

            fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
                match resolve_option(option)? {
                    CommandDataOptionValue::Integer(value) => Self::new(
                        <$t>::try_from(value).map_err(|error| ResolveError::Other(error.into()))?,
                    )
                    .ok_or(ResolveError::IntegerRangeError),
                    _ => Err(ResolveError::InvalidType),
                }
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

            fn describe(option: &mut $crate::serenity::builder::CreateApplicationCommandOption, _l10n: &$crate::l10n::L10n) {
                $min.map(|min| option.min_number_value(min));
                $max.map(|max| option.max_number_value(max));
            }

            fn resolve(
                option: std::option::Option<$crate::serenity::model::application::interaction::application_command::CommandDataOption>
            ) -> $crate::resolve::ResolveResult<Self> {
                <::std::primitive::f64 as $crate::resolve::Resolve>::resolve(option).and_then(Self::try_from)
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

            fn describe(option: &mut $crate::serenity::builder::CreateApplicationCommandOption, _l10n: &$crate::l10n::L10n) {
                $min.map(|min| option.min_length(min));
                $max.map(|max| option.max_length(max));
            }

            fn resolve(
                option: std::option::Option<$crate::serenity::model::application::interaction::application_command::CommandDataOption>
            ) -> $crate::resolve::ResolveResult<Self> {
                <::std::string::String as $crate::resolve::Resolve>::resolve(option).and_then(Self::try_from)
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

pub struct Choice {
    pub name: String,
    pub value: String,
}

pub trait Choices: Sized {
    /// Only used to distinguish different types in the l10n file.
    fn name() -> String;
    fn choices() -> Vec<Choice>;
    fn resolve(choice: String) -> Option<Self>;
}

impl<T: Choices> Resolve for T {
    const KIND: CommandOptionType = CommandOptionType::String;

    fn resolve(option: Option<CommandDataOption>) -> ResolveResult<Self> {
        T::resolve(String::resolve(option)?).ok_or(ResolveError::InvalidChoice)
    }

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        String::describe(option, l10n);
        for Choice { name, value } in Self::choices() {
            l10n.describe_string_choice(&Self::name(), &name, &value, option);
        }
    }
}

pub fn resolve_command_options(command_data: &CommandData) -> &[CommandDataOption] {
    match command_data.options.as_slice() {
        [group]
            if group.kind == CommandOptionType::SubCommand
                || group.kind == CommandOptionType::SubCommandGroup =>
        {
            match group.options.as_slice() {
                [subcommand] if subcommand.kind == CommandOptionType::SubCommand => {
                    &subcommand.options
                }
                _ => &group.options,
            }
        }
        _ => &command_data.options,
    }
}
