pub mod option;

use serenity::all::{CommandInteraction, Context, CreateAutocompleteResponse, CreateCommand};
use thiserror::Error;
pub use tranquil_macros::Command;

use super::error::Result;

pub trait Command: Run + Sized {
    /// Should be the same as [`CreateCommand::name`].
    ///
    /// Used to match against commands returned from the endpoint upon creation.
    const NAME: &str;

    /// A separate type containing all commands that have autocomplete options.
    ///
    /// [`NoAutocomplete`] if none of the commands have any autocompleted option.
    type Autocomplete: Autocomplete;

    /// Returns a builder for this command which can be sent to Discord.
    fn create() -> CreateCommand;

    fn resolve_command(data: &mut CommandInteraction) -> Result<Self>;

    fn resolve_autocomplete(data: &mut CommandInteraction) -> Result<Self::Autocomplete>;
}

/// Marks a command group inside a [`Command`].
///
/// The [`Group`] itself is not actually callable as a command. For that reason [`Group`] is a
/// **never type** that cannot be instantiated. These [`Group`] variants act solely as markers to
/// provide a description (and localizations) for the group itself.
///
/// Commands following the [`Group`] will be part of that group. The group concludes automatically
/// once a command does not have the name of the [`Group`] variant as a prefix. This is usually
/// sufficient, but a group can also be concluded manually via the `#[tranquil(end_group)]`
/// attribute as shown in the example below.
///
/// ```
/// # use tranquil::interaction::command::{Command, Group};
/// /// Pings users.
/// #[derive(Command)]
/// enum Ping {
///     /// Pings all users on the server.
///     All,
///
///     /// Pings a user.
///     User(Group),
///     /// Pings a specific user.
///     UserByName { user: UserId },
///     /// Pings a random user on the server.
///     UserRandom,
///
///     /// A separate `user-foo` command rather than a `foo` command in the `user` group.
///     #[tranquil(end_group)]
///     UserFoo,
/// }
///
/// # let ping = Ping::All;
/// match ping {
///     Ping::All => {}
///     // omitting Ping::User is still an exhaustive match due to Group being a never type
///     Ping::UserByName { .. } => {}
///     Ping::UserRandom => {}
///     Ping::UserFoo => {}
/// }
/// ```
pub enum Group {}

pub trait Run {
    fn run(
        self,
        ctx: Context,
        interaction: CommandInteraction,
    ) -> impl Future<Output = Result<()>> + Send;
}

pub trait Autocomplete {
    fn autocomplete(
        self,
        ctx: Context,
        interaction: CommandInteraction,
    ) -> impl Future<Output = Result<CreateAutocompleteResponse>> + Send;
}

pub enum NoAutocomplete {}

impl Autocomplete for NoAutocomplete {
    async fn autocomplete(
        self,
        _ctx: Context,
        _interaction: CommandInteraction,
    ) -> Result<CreateAutocompleteResponse> {
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
