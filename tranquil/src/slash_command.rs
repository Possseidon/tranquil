use std::{pin::Pin, sync::Arc};

use futures::Future;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

use crate::{module::Module, AnyResult};

type SlashCommandFunction<M> = Box<
    dyn Fn(
            Context,
            ApplicationCommandInteraction,
            Arc<M>,
        ) -> Pin<Box<dyn Future<Output = AnyResult<()>> + Send + Sync>>
        + Send
        + Sync,
>;

pub struct SlashCommand<M: Module> {
    name: String,
    function: SlashCommandFunction<M>,
    option_builders: Vec<fn() -> CreateApplicationCommandOption>,
    module: Arc<M>,
}

impl<M: Module> SlashCommand<M> {
    pub fn new(
        name: impl Into<String>,
        function: SlashCommandFunction<M>,
        option_builders: impl Into<Vec<fn() -> CreateApplicationCommandOption>>,
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
pub trait SlashCommandImpl: Send + Sync {
    fn name(&self) -> &str;
    fn create_application_command(&self, command: &mut CreateApplicationCommand);
    async fn run(&self, ctx: Context, interaction: ApplicationCommandInteraction) -> AnyResult<()>;
}

#[async_trait]
impl<M: Module> SlashCommandImpl for SlashCommand<M> {
    fn name(&self) -> &str {
        &self.name
    }

    fn create_application_command(&self, command: &mut CreateApplicationCommand) {
        for option_builder in &self.option_builders {
            let mut option = option_builder();
            option.description("TODO");
            command.add_option(option);
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

pub type SlashCommands = Vec<Box<dyn SlashCommandImpl>>;

pub trait SlashCommandProvider {
    fn slash_commands(self: Arc<Self>) -> SlashCommands;
}
