pub mod interaction;
pub mod locale;

// Re-exported for tranquil-macros
pub use anyhow;
// Re-exported for tranquil-macros
pub use serenity;
use serenity::{
    all::{
        CommandInteraction, ComponentInteraction, Context, Framework, FullEvent, Interaction,
        ModalInteraction, Ready,
    },
    async_trait,
};
use thiserror::Error;
use tracing::error;

pub struct Tranquil {
    // TODO
}

#[async_trait]
impl Framework for Tranquil {
    async fn dispatch(&self, ctx: Context, event: FullEvent) {
        match event {
            FullEvent::Ready { data_about_bot } => self.ready(ctx, data_about_bot).await,
            FullEvent::InteractionCreate { interaction } => {
                self.interaction(ctx, interaction).await
            }
            _ => {}
        }
    }
}

impl Tranquil {
    pub fn new() -> Self {
        Self {}
    }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        // self.synced_commands.get_or_init(async || {
        //     let commands = Command::set_global_commands(&ctx, take(&mut self.create_commands))
        //         .await
        //         .unwrap(); // TODO: turn into Err
        //     assert_eq!(commands.len(), self.command_fns.len()); // TODO: turn into Err
        //     commands
        //         .into_iter()
        //         .zip(take(&mut self.command_fns))
        //         .map(|(command, fns)| {
        //             assert!(command.name == fns.name);
        //             (command.id, fns)
        //         })
        //         .collect()
        // });

        // register commands globally, manually for all connected or for specific guilds

        // Command::set_global_commands(&ctx, commands);

        // for guild in data_about_bot.guilds {
        //     guild
        //         .id
        //         .set_commands(&ctx)
        //         .await
        //         .inspect_err(|error| {
        //             eprintln!("Failed updating commands {error}");
        //         })
        //         .ok();
        // }
    }

    async fn interaction(&self, ctx: Context, interaction: Interaction) {
        let result = match interaction {
            Interaction::Command(interaction) => self
                .command_interaction(ctx, interaction, false)
                .await
                .map_err(InteractionError::Command),
            Interaction::Autocomplete(interaction) => self
                .command_interaction(ctx, interaction, true)
                .await
                .map_err(InteractionError::Command),
            Interaction::Component(interaction) => self
                .component_interaction(ctx, interaction)
                .await
                .map_err(InteractionError::Component),
            Interaction::Modal(interaction) => self
                .modal_interaction(ctx, interaction)
                .await
                .map_err(InteractionError::Modal),
            _ => Ok(()),
        };

        if let Err(error) = result {
            error!("Error while handling interaction: {error:#?}");
        }
    }

    async fn command_interaction(
        &self,
        ctx: Context,
        interaction: CommandInteraction,
        autocomplete: bool,
    ) -> Result<(), CommandInteractionError> {
        Err(CommandInteractionError::FooBar)
    }

    async fn component_interaction(
        &self,
        ctx: Context,
        interaction: ComponentInteraction,
    ) -> Result<(), ComponentInteractionError> {
        Ok(())
    }

    async fn modal_interaction(
        &self,
        ctx: Context,
        interaction: ModalInteraction,
    ) -> Result<(), ModalInteractionError> {
        Ok(())
    }
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
pub enum CommandInteractionError {
    #[error("foo bar")]
    FooBar,
}

#[derive(Debug, Error)]
pub enum ComponentInteractionError {}

#[derive(Debug, Error)]
pub enum ModalInteractionError {}
