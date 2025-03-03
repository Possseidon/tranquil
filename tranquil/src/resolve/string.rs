use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommandOption, model::application::command::CommandOptionType,
};

use super::{Resolve, ResolveContext, ResolveError, ResolveResult};
use crate::l10n::L10n;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Choice {
    pub name: String,
    pub value: String,
}

pub use tranquil_macros::Choices;

pub trait Choices: Sized {
    /// Only used to distinguish different types in the l10n file.
    fn name() -> String;
    fn choices() -> Vec<Choice>;
    fn resolve(choice: String) -> Option<Self>;
}

#[async_trait]
impl<T: Choices> Resolve for T {
    const KIND: CommandOptionType = CommandOptionType::String;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        T::resolve(String::resolve(ctx).await?).ok_or(ResolveError::InvalidChoice)
    }

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        String::describe(option, l10n);
        for Choice { name, value } in Self::choices() {
            l10n.describe_string_choice(&Self::name(), &name, &value, option);
        }
    }
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

        #[$crate::async_trait]
        impl $crate::resolve::Resolve for $name {
            const KIND: $crate::serenity::model::application::command::CommandOptionType =
                <::std::string::String as $crate::resolve::Resolve>::KIND;

            fn describe(option: &mut $crate::serenity::builder::CreateApplicationCommandOption, _l10n: &$crate::l10n::L10n) {
                $min.map(|min| option.min_length(min));
                $max.map(|max| option.max_length(max));
            }

            async fn resolve(ctx: $crate::resolve::ResolveContext) -> $crate::resolve::ResolveResult<Self> {
                <::std::string::String as $crate::resolve::Resolve>::resolve(ctx).await.and_then(Self::try_from)
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
