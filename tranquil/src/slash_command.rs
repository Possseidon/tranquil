use std::{pin::Pin, sync::Arc};

use futures::Future;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        application::command::CommandOptionType,
        application::interaction::application_command::ApplicationCommandInteraction,
    },
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

pub struct ParameterInfo {
    pub name: String,
    pub kind: CommandOptionType,
    pub required: bool,
}

fn create_application_command<'a>(
    parameter_info: impl IntoIterator<Item = &'a ParameterInfo>,
    command: &mut CreateApplicationCommand,
) {
    for info in parameter_info {
        command.create_option(|option| {
            option
                .name(&info.name)
                .description("TODO")
                .kind(info.kind)
                .required(info.required)
        });
    }
}

pub struct SlashCommand<M: Module> {
    name: String,
    function: SlashCommandFunction<M>,
    parameter_info: Vec<ParameterInfo>,
    module: Arc<M>,
}

impl<M: Module> SlashCommand<M> {
    pub fn new(
        name: impl Into<String>,
        function: SlashCommandFunction<M>,
        parameter_info: impl Into<Vec<ParameterInfo>>,
        module: Arc<M>,
    ) -> Self {
        Self {
            name: name.into(),
            function,
            parameter_info: parameter_info.into(),
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
        // TODO: Only call parameter_info once
        create_application_command(&self.parameter_info, command);
    }

    async fn run(&self, ctx: Context, interaction: ApplicationCommandInteraction) -> AnyResult<()> {
        (self.function)(ctx, interaction, self.module.clone()).await
        // TODO: return a different type of error so e.g. invalid parameters can automatically be reported nicely like here:

        /*
        match parameter_from_interaction(&self.parameter_info, &interaction) {
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
