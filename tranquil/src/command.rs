use std::{pin::Pin, sync::Arc};

use futures::Future;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

use crate::{l10n::TranslatedCommands, module::Module, AnyResult};

type CommandFunction<M> = Box<
    dyn Fn(
            Context,
            ApplicationCommandInteraction,
            Arc<M>,
        ) -> Pin<Box<dyn Future<Output = AnyResult<()>> + Send + Sync>>
        + Send
        + Sync,
>;

pub struct Command<M: Module> {
    name: String,
    function: CommandFunction<M>,
    option_builders: Vec<fn(&TranslatedCommands) -> CreateApplicationCommandOption>,
    module: Arc<M>,
}

impl<M: Module> Command<M> {
    pub fn new(
        name: impl Into<String>,
        function: CommandFunction<M>,
        option_builders: impl Into<Vec<fn(&TranslatedCommands) -> CreateApplicationCommandOption>>,
        module: Arc<M>,
    ) -> Self {
        Self {
            name: name.into(),
            function,
            option_builders: option_builders.into(),
            module,
        }
    }
}

#[async_trait]
pub trait CommandImpl: Send + Sync {
    fn name(&self) -> &str;
    fn create_application_command(
        &self,
        translated_commands: &TranslatedCommands,
        command: &mut CreateApplicationCommand,
    );
    async fn run(&self, ctx: Context, interaction: ApplicationCommandInteraction) -> AnyResult<()>;
}

#[async_trait]
impl<M: Module> CommandImpl for Command<M> {
    fn name(&self) -> &str {
        &self.name
    }

    fn create_application_command(
        &self,
        translated_commands: &TranslatedCommands,
        command: &mut CreateApplicationCommand,
    ) {
        for option_builder in &self.option_builders {
            command.add_option(option_builder(translated_commands));
        }
    }

    async fn run(&self, ctx: Context, interaction: ApplicationCommandInteraction) -> AnyResult<()> {
        (self.function)(ctx, interaction, self.module.clone()).await
        // TODO: return a different type of error so e.g. invalid parameters can automatically be reported nicely like here:

        /*
        match parameter_from_interaction(&self.command_options, &interaction) {
            Ok(parameter) => (self.function)(ctx, interaction, parameter).await,
            Err(invalid_parameters) => {
                interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response.interaction_response_data(|data| {
                            data.ephemeral(true).embed(|embed| {
                                embed
                                    .title(format!(
                                        "Invalid parameters to `/{}`",
                                        interaction.data.name
                                    ))
                                    .color(colors::css::DANGER)
                                    .fields(invalid_parameters.iter().map(|invalid_parameter| {
                                        (
                                            format!("{}", &invalid_parameter.name),
                                            format!("{}", invalid_parameter.error),
                                            false,
                                        )
                                    }))
                            })
                        })
                    })
                    .await
            }
        }
        */
    }
}

pub struct CommandContext {
    pub ctx: Context,
    pub interaction: ApplicationCommandInteraction,
}

pub type Commands = Vec<Box<dyn CommandImpl>>;

pub trait CommandProvider {
    fn commands(self: Arc<Self>) -> Commands;
}
