use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use futures::Future;
use serenity::{
    builder::CreateApplicationCommandOption,
    model::application::{
        command::CommandOptionType, interaction::application_command::CommandDataOption,
    },
};

use crate::{
    context::autocomplete::AutocompleteCtx,
    l10n::L10n,
    resolve::{Resolve, ResolveContext, ResolveResult},
};

pub(crate) type AutocompleteFunction<M> = Box<
    dyn Fn(
            Arc<M>,
            AutocompleteCtx,
            Vec<CommandDataOption>,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Autocomplete<T>(pub T);

#[async_trait]
impl<T: Resolve> Resolve for Autocomplete<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = T::REQUIRED;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        T::describe(option, l10n);
        option.set_autocomplete(true);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        Ok(Autocomplete(T::resolve(ctx).await?))
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Focusable<T> {
    pub has_focus: bool,
    pub current: T,
}

#[async_trait]
impl<T: Resolve> Resolve for Focusable<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = T::REQUIRED;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        T::describe(option, l10n);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        Ok(Focusable {
            has_focus: ctx.option.as_ref().map_or(false, |option| option.focused),
            current: T::resolve(ctx).await?,
        })
    }
}
