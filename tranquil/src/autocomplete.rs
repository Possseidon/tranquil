use std::{pin::Pin, sync::Arc};

use futures::Future;
use serenity::{
    builder::{CreateApplicationCommandOption, CreateAutocompleteResponse},
    client::Context,
    model::application::{
        command::CommandOptionType,
        interaction::{
            application_command::CommandDataOption, autocomplete::AutocompleteInteraction,
        },
    },
};

use crate::{
    resolve::{Resolve, ResolveResult},
    AnyResult,
};

pub struct AutocompleteContext {
    pub bot: Context,
    pub interaction: AutocompleteInteraction,
}

impl AutocompleteContext {
    pub async fn create_response<F>(&self, f: F) -> serenity::Result<()>
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse,
    {
        self.interaction
            .create_autocomplete_response(&self.bot, f)
            .await
    }
}

pub(crate) type AutocompleteFunction<M> = Box<
    dyn Fn(
            Arc<M>,
            AutocompleteContext,
        ) -> Pin<Box<dyn Future<Output = AnyResult<()>> + Send + Sync>>
        + Send
        + Sync,
>;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Autocomplete<T>(pub T);

impl<T: Resolve> Resolve for Autocomplete<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = T::REQUIRED;

    fn describe(option: &mut CreateApplicationCommandOption) {
        T::describe(option);
        option.set_autocomplete(true);
    }

    fn resolve(option: Option<&CommandDataOption>) -> ResolveResult<Self> {
        Ok(Autocomplete(T::resolve(option)?))
    }
}

pub struct Focusable<T> {
    pub has_focus: bool,
    pub current: T,
}

impl<T: Resolve> Resolve for Focusable<T> {
    const KIND: CommandOptionType = T::KIND;
    const REQUIRED: bool = T::REQUIRED;

    fn describe(option: &mut CreateApplicationCommandOption) {
        T::describe(option);
    }

    fn resolve(option: Option<&CommandDataOption>) -> ResolveResult<Self> {
        Ok(Focusable {
            has_focus: option.map_or(false, |option| option.focused),
            current: T::resolve(option)?,
        })
    }
}