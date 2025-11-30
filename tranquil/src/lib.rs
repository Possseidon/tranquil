pub(crate) mod display;
mod gateway;
mod http;
pub mod id;
pub mod interaction;
pub mod locale;
pub mod markdown;
pub mod model;
pub mod snowflake;

use std::{collections::HashMap, pin::Pin, sync::Mutex};

use thiserror::Error;
use tracing::error;

use crate::interaction::{
    command::{Autocomplete, Command, Run},
    error::Error,
};

#[derive(Default)]
pub struct Tranquil<B> {
    bot: B,
    create_commands: Mutex<Vec<CreateCommand>>,
    command_handlers: HashMap<&'static str, CommandHandler<B>>,
    // component_handlers: HashMap<&'static str, ComponentHandler<B>>,
    // modal_handlers: HashMap<&'static str, ModalHandler<B>>,
}

// #[async_trait]
// impl<B: Send + Sync> Framework for Tranquil<B> {
//     async fn init(&mut self, _client: &serenity::Client) {}

//     async fn dispatch(&self, ctx: Context, event: FullEvent) {
//         match event {
//             FullEvent::Ready { data_about_bot } => self.ready(ctx, data_about_bot).await,
//             FullEvent::InteractionCreate { interaction } => {
//                 self.interaction(ctx, interaction).await
//             }
//             _ => {}
//         }
//     }
// }

impl<T> Tranquil<T> {
    pub fn new(bot: T) -> Self {
        Self {
            bot,
            create_commands: Default::default(),
            command_handlers: Default::default(),
        }
    }

    /// Registers a [`Command`].
    ///
    /// # Panics
    ///
    /// Panics if a command with this name was already registered.
    pub fn command<C: Command>(mut self) -> Self
    where
        T: Sync + Run<C> + Autocomplete<C::Autocomplete>,
    {
        let new = self
            .command_handlers
            .insert(
                C::NAME,
                CommandHandler {
                    interact: |bot, ctx, mut interaction| {
                        Box::pin(async move {
                            bot.run(C::resolve(&mut interaction)?, ctx, interaction)
                                .await
                        })
                    },
                    autocomplete: |bot, ctx, mut interaction| {
                        Box::pin(async move {
                            bot.autocomplete(
                                C::resolve_autocomplete(&mut interaction)?,
                                ctx,
                                interaction,
                            )
                            .await
                        })
                    },
                },
            )
            .is_none();

        assert!(new, "command {:?} already exists", C::NAME);

        // self.create_commands.get_mut().unwrap().push(C::create());

        self
    }

    // pub fn component<C: Component>(mut self) -> Self {
    //     let new = self.component_handlers.insert(k, v).is_none();
    //     assert!(new, "component {:?} already exists", C::CUSTOM_ID);
    //     self
    // }

    // pub fn modal<M: Modal>(mut self) -> Self {
    //     let new = self.modal_handlers.insert(k, v).is_none();
    //     assert!(new, "modal {:?} already exists", M::CUSTOM_ID);
    //     self
    // }

    // async fn ready(&self, ctx: Context, data_about_bot: Ready) {
    //     let commands = take(&mut *self.create_commands.lock().unwrap());

    //     // TODO: Add custom update logic again:
    //     // - update commands globally
    //     // - only for connected guilds (current)
    //     // - only for specific guilds (usually from .env)

    //     for guild in data_about_bot.guilds {
    //         guild.id.set_commands(&ctx, commands.clone()).await.unwrap();
    //     }

    //     // let commands = serenity::all::Command::set_global_commands(&ctx, commands)
    //     //     .await
    //     //     .unwrap();
    // }

    // async fn interaction(&self, ctx: Context, interaction: Interaction) {
    //     let result = match interaction {
    //         Interaction::Command(interaction) => self
    //             .command_interaction(ctx, interaction, false)
    //             .await
    //             .map_err(InteractionError::Command),
    //         Interaction::Autocomplete(interaction) => self
    //             .command_interaction(ctx, interaction, true)
    //             .await
    //             .map_err(InteractionError::Command),
    //         Interaction::Component(interaction) => self
    //             .component_interaction(ctx, interaction)
    //             .await
    //             .map_err(InteractionError::Component),
    //         Interaction::Modal(interaction) => self
    //             .modal_interaction(ctx, interaction)
    //             .await
    //             .map_err(InteractionError::Modal),
    //         _ => Ok(()),
    //     };

    //     if let Err(error) = result {
    //         error!("Error while handling interaction: {error:#?}");
    //     }
    // }

    // async fn command_interaction(
    //     &self,
    //     ctx: Context,
    //     mut interaction: CommandInteraction,
    //     autocomplete: bool,
    // ) -> Result<(), CommandInteractionError> {
    //     // TODO: get rid of all unwraps
    //     let command_handler = self.command_handlers.get(&*interaction.data.name).unwrap();
    //     if autocomplete {
    //         let interaction_id = interaction.id;
    //         // This token is only used for responding to the interaction, so it shouldn't be
    // needed         // by the user-handler and can be taken.
    //         let interaction_token = take(&mut interaction.token);
    //         (command_handler.autocomplete)(&self.bot, &ctx, interaction)
    //             .await
    //             .unwrap()
    //             .execute(ctx, (interaction_id, &interaction_token))
    //             .await
    //             .unwrap();
    //     } else {
    //         (command_handler.interact)(&self.bot, &ctx, interaction)
    //             .await
    //             .unwrap();
    //     }
    //     Ok(())
    // }

    // async fn component_interaction(
    //     &self,
    //     ctx: Context,
    //     interaction: ComponentInteraction,
    // ) -> Result<(), ComponentInteractionError> {
    //     // let component_handler = self
    //     //     .component_handlers
    //     //     .get(interaction.data.custom_id)
    //     //     .unwrap();
    //     Ok(())
    // }

    // async fn modal_interaction(
    //     &self,
    //     ctx: Context,
    //     interaction: ModalInteraction,
    // ) -> Result<(), ModalInteractionError> {
    //     // let modal_handler = self.modal_handlers.get(interaction.data.custom_id).unwrap();
    //     Ok(())
    // }
}

#[derive(Debug, Error)]
pub enum InteractionError {
    #[error(transparent)]
    Command(CommandInteractionError),
    #[error(transparent)]
    Component(ComponentInteractionError),
    #[error(transparent)]
    Modal(ModalInteractionError),
}

#[derive(Debug, Error)]
pub enum CommandInteractionError {}

#[derive(Debug, Error)]
pub enum ComponentInteractionError {}

#[derive(Debug, Error)]
pub enum ModalInteractionError {}

struct CommandHandler<B> {
    interact: ResolveAndRun<B, ()>,
    autocomplete: ResolveAndRun<B, CreateAutocompleteResponse>,
}

type ResolveAndRun<B, R> =
    for<'a> fn(
        &'a B,
        &'a Context,
        CommandInteraction,
    ) -> Pin<Box<dyn Future<Output = Result<R, Error>> + Send + 'a>>;

struct CreateCommand;
struct Context;
struct CreateAutocompleteResponse;
struct CommandInteraction;
