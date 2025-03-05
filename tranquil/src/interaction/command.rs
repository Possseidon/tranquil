pub mod option;

use anyhow::Result;
use serenity::all::{CommandInteraction, Context, CreateAutocompleteResponse, CreateCommand};
use thiserror::Error;
pub use tranquil_macros::Command;

pub trait Command: Run + Sized {
    /// Should be the same as [`CreateCommand::name`].
    ///
    /// Used to match against commands returned from the endpoint upon creation.
    const NAME: &str;

    type Autocomplete: Autocomplete;

    fn create_command() -> CreateCommand;

    fn resolve_autocomplete(data: &mut CommandInteraction) -> Result<Self::Autocomplete>;

    fn resolve_command(data: &mut CommandInteraction) -> Result<Self>;
}

pub trait Run {
    fn run(
        self,
        ctx: Context,
        interaction: CommandInteraction,
    ) -> impl Future<Output = Result<(), RunError>> + Send;
}

pub trait Autocomplete {
    fn autocomplete(
        self,
        ctx: Context,
        interaction: CommandInteraction,
    ) -> impl Future<Output = Result<CreateAutocompleteResponse, AutocompleteError>> + Send;
}

pub enum NoAutocomplete {}

impl Autocomplete for NoAutocomplete {
    async fn autocomplete(
        self,
        _ctx: Context,
        _interaction: CommandInteraction,
    ) -> Result<CreateAutocompleteResponse, AutocompleteError> {
        match self {}
    }
}

#[derive(Debug, Error)]
pub enum CommandResolveError {}

#[derive(Debug, Error)]
pub enum AutocompleteResolveError {
    #[error("the command does not support autocompletion")]
    NoAutocomplete,
}

#[derive(Debug, Error)]
pub enum RunError {}

#[derive(Debug, Error)]
pub enum AutocompleteError {}

// pub(super) struct CommandFns {
//     pub(super) name: &'static str,
//     pub(super) state: Arc<dyn Any>,
//     pub(super) interact: CommandFn<()>,
//     pub(super) autocomplete: CommandFn<CreateAutocompleteResponse>,
// }

// impl CommandFns {
//     pub(super) fn new<T: Command>(ctx: &Context) -> Self {
//         Self {
//             name: T::NAME,
//             state: <T as Run>::State::resolve(ctx),
//             autocomplete: |ctx, interaction, data| {
//                 Box::pin(async move {
//                     T::resolve_autocomplete(data)
//                         .autocomplete(ctx, interaction)
//                         .await
//                 })
//             },
//             interact: |ctx, interaction, data| {
//                 Box::pin(async move { T::resolve_command(data).execute(ctx, interaction).await })
//             },
//         }
//     }
// }

// type CommandFn<T> = fn(Context, CommandInteraction, CommandData) -> BoxFuture<'static,
// Result<T>>;
