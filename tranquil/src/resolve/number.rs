use serenity::{
    async_trait,
    model::application::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
};

use super::{resolve_option, Resolve, ResolveContext, ResolveError, ResolveResult};

#[async_trait]
impl Resolve for f32 {
    const KIND: CommandOptionType = CommandOptionType::Number;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::Number(value) => Ok(value as _),
            _ => Err(ResolveError::InvalidType),
        }
    }
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

        #[$crate::serenity::async_trait]
        impl $crate::resolve::Resolve for $name {
            const KIND: $crate::serenity::model::application::command::CommandOptionType =
                <::std::primitive::f64 as $crate::resolve::Resolve>::KIND;

            fn describe(option: &mut $crate::serenity::builder::CreateApplicationCommandOption, _l10n: &$crate::l10n::L10n) {
                $min.map(|min| option.min_number_value(min));
                $max.map(|max| option.max_number_value(max));
            }

            async fn resolve(ctx: $crate::resolve::ResolveContext) -> $crate::resolve::ResolveResult<Self> {
                <::std::primitive::f64 as $crate::resolve::Resolve>::resolve(ctx).await.and_then(Self::try_from)
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
