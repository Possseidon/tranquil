use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommandOption,
    model::application::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
};

use super::{resolve_option, Resolve, ResolveContext, ResolveError, ResolveResult};
use crate::l10n::L10n;

pub const DISCORD_MIN_INTEGER: i64 = -9007199254740991;
pub const DISCORD_MAX_INTEGER: i64 = 9007199254740991;

macro_rules! impl_resolve_for_integer {
    ($($t:ty),* $(,)?) => { $(
        #[async_trait]
        impl Resolve for $t {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            #[allow(irrefutable_let_patterns)]
            fn describe(option: &mut CreateApplicationCommandOption, _l10n: &L10n) {
                if let Ok(min) = i64::try_from(<$t>::MIN) {
                    option.min_int_value(min.max(DISCORD_MIN_INTEGER));
                }
                if let Ok(max) = i64::try_from(<$t>::MAX) {
                    option.max_int_value(max.min(DISCORD_MAX_INTEGER));
                }
            }

            async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
                match resolve_option(ctx.option)? {
                    CommandDataOptionValue::Integer(value) => {
                        Ok(<$t>::try_from(value)?)
                    }
                    _ => Err(ResolveError::InvalidType),
                }
            }
        }
    )* };
}

impl_resolve_for_integer!(i8, i16, i32, i128, isize, u8, u16, u32, u64, u128, usize);

macro_rules! impl_resolve_for_bounded_integer {
    ($( $t:ty => $b:ident ),* $(,)?) => { $(
        #[async_trait]
        impl<const MIN: $t, const MAX: $t> Resolve for bounded_integer::$b<MIN, MAX> {
            const KIND: CommandOptionType = CommandOptionType::Integer;

            #[allow(irrefutable_let_patterns)]
            fn describe(option: &mut CreateApplicationCommandOption, _l10n: &L10n) {
                if let Ok(min) = i64::try_from(<$t>::MIN) {
                    option.min_int_value(min.max(DISCORD_MIN_INTEGER));
                }
                if let Ok(max) = i64::try_from(<$t>::MAX) {
                    option.max_int_value(max.min(DISCORD_MAX_INTEGER));
                }
            }

            async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
                match resolve_option(ctx.option)? {
                    CommandDataOptionValue::Integer(value) => Self::new(
                        <$t>::try_from(value)?,
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
