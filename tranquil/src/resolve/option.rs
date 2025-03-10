use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommandOption, model::application::command::CommandOptionType,
};

use super::{Resolve, ResolveContext, ResolveResult};
use crate::l10n::L10n;

#[async_trait]
impl<T: Resolve> Resolve for Option<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = false;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        T::describe(option, l10n);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        Ok(match ctx.option {
            Some(_) => Some(T::resolve(ctx).await?),
            None => None,
        })
    }
}
