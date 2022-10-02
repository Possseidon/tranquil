use futures::{future::join_all, join};
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::{Context, EventHandler},
    framework::StandardFramework,
    http::Http,
    model::{
        application::{
            command::{Command, CommandOptionType},
            interaction::{
                application_command::{
                    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
                },
                Interaction,
            },
        },
        gateway::{GatewayIntents, Ready},
        guild::Guild,
        id::GuildId,
    },
    utils::colours as colors,
};
use std::{
    error::Error,
    fmt,
    future::Future,
    iter::zip,
    pin::Pin,
    sync::atomic::{self, AtomicBool},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplicationCommandUpdate {
    #[default]
    Global,
    Connected,
    Only(Vec<GuildId>),
}

type SlashCommandFunction<T> = Box<
    dyn Fn(
            Context,
            ApplicationCommandInteraction,
            T,
        ) -> Pin<Box<dyn Future<Output = serenity::Result<()>> + Send + Sync>>
        + Send
        + Sync,
>;

pub struct ParameterInfo<T> {
    name: String,
    description: String,
    kind: CommandOptionType,
    // TODO: Localization
    fill_parameter: Box<dyn Fn(&CommandDataOption, &mut T) -> AnyResult<()> + Send + Sync>,
}

#[derive(Debug)]
pub enum ResolveError {
    InvalidType,
    Unresolvable,
}

impl Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ResolveError::InvalidType => "parameter has invalid type",
                ResolveError::Unresolvable => "paremeter is unresolvable",
            }
        )
    }
}

pub type AnyError = Box<dyn Error + Send + Sync>;
pub type AnyResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub trait Resolvable
where
    Self: Sized,
{
    fn kind() -> CommandOptionType;

    fn resolve(option: &CommandDataOption) -> AnyResult<Self> {
        match option.resolved.as_ref() {
            Some(value) => Self::resolve_value(value),
            None => Err(ResolveError::Unresolvable)?,
        }
    }

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self>;
}

impl Resolvable for i64 {
    fn kind() -> CommandOptionType {
        CommandOptionType::Integer
    }

    fn resolve_value(value: &CommandDataOptionValue) -> AnyResult<Self> {
        match value {
            CommandDataOptionValue::Integer(value) => Ok(*value),
            _ => Err(ResolveError::InvalidType)?,
        }
    }
}

impl<T> ParameterInfo<T> {
    pub fn new<R: Resolvable>(
        name: impl ToString,
        description: impl ToString,
        get_parameter: impl Fn(&mut T) -> &mut R + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            kind: R::kind(),
            fill_parameter: Box::new(move |option, this| {
                R::resolve(option).map(|resolved| *get_parameter(this) = resolved)
            }),
        }
    }
}

struct InvalidParameter {
    name: String,
    error: AnyError,
}

pub trait SlashCommandParameter: Send + Sync
where
    Self: Default,
{
    fn parameter_info() -> Vec<ParameterInfo<Self>>;
}

fn create_application_command<'a, T: SlashCommandParameter + 'a>(
    parameter_info: impl IntoIterator<Item = &'a ParameterInfo<T>>,
    command: &mut CreateApplicationCommand,
) {
    for info in parameter_info {
        command.create_option(|option| {
            option
                .name(&info.name)
                .description(&info.description)
                .kind(info.kind)
        });
    }
}

fn parameter_from_interaction<'a, T: SlashCommandParameter + 'a>(
    parameter_info: impl IntoIterator<Item = &'a ParameterInfo<T>>,
    interaction: &ApplicationCommandInteraction,
) -> Result<T, Vec<InvalidParameter>> {
    let mut parameter = T::default();
    let invalid_parameters = zip(interaction.data.options.iter(), parameter_info.into_iter())
        .filter_map(|(option, info)| {
            (info.fill_parameter)(option, &mut parameter)
                .err()
                .map(|error| InvalidParameter {
                    name: option.name.clone(),
                    error,
                })
        })
        .collect::<Vec<_>>();

    if invalid_parameters.is_empty() {
        Ok(parameter)
    } else {
        Err(invalid_parameters)
    }
}

pub struct SlashCommand<T: SlashCommandParameter> {
    name: String,
    description: String,
    function: SlashCommandFunction<T>,
    parameter_info: Vec<ParameterInfo<T>>,
}

impl<T: SlashCommandParameter> SlashCommand<T> {
    pub fn new<F>(
        name: impl ToString,
        description: impl ToString,
        command: impl Fn(Context, ApplicationCommandInteraction, T) -> F + Send + Sync + 'static,
    ) -> Self
    where
        F: Future<Output = serenity::Result<()>> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            function: Box::new(move |ctx, interaction, parameter| {
                Box::pin(command(ctx, interaction, parameter))
            }),
            parameter_info: T::parameter_info(),
        }
    }
}

#[async_trait]
pub trait SlashCommandImpl: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn create_application_command(&self, command: &mut CreateApplicationCommand);
    async fn run(
        &self,
        ctx: Context,
        interaction: ApplicationCommandInteraction,
    ) -> serenity::Result<()>;
}

#[async_trait]
impl<T: SlashCommandParameter> SlashCommandImpl for SlashCommand<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn create_application_command(&self, command: &mut CreateApplicationCommand) {
        // TODO: Only call parameter_info once
        create_application_command(&self.parameter_info, command);
    }

    async fn run(
        &self,
        ctx: Context,
        interaction: ApplicationCommandInteraction,
    ) -> serenity::Result<()> {
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
                                            invalid_parameter.name.to_string(),
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
    }
}

pub type SlashCommands = Vec<Box<dyn SlashCommandImpl>>;

#[async_trait]
pub trait Module: Send + Sync {
    fn intents(&self) -> GatewayIntents {
        GatewayIntents::empty()
    }

    fn slash_commands(&self) -> SlashCommands {
        SlashCommands::default()
    }
}

fn merge_intents(modules: &[Box<dyn Module>]) -> GatewayIntents {
    modules
        .iter()
        .fold(GatewayIntents::empty(), |intents, module| {
            intents | module.intents()
        })
}

pub struct Bot {
    already_connected: AtomicBool,
    application_command_update: Option<ApplicationCommandUpdate>,
    slash_commands: SlashCommands,
    modules: Vec<Box<dyn Module>>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            already_connected: Default::default(),
            application_command_update: Some(ApplicationCommandUpdate::default()),
            slash_commands: Default::default(),
            modules: Default::default(),
        }
    }

    pub fn application_command_update(
        mut self,
        application_command_update: Option<ApplicationCommandUpdate>,
    ) -> Self {
        self.application_command_update = application_command_update;
        self
    }

    pub fn register(mut self, module: Box<dyn Module>) -> Self {
        self.slash_commands.append(&mut module.slash_commands());
        self.modules.push(module);
        self
    }

    pub async fn run(self, token: impl AsRef<str>) -> Result<(), Box<dyn Error>> {
        // TODO: Token validation doesn't work, because of the middle "timestamp" part not always being valid base64.
        // validate_token(&token).map_err(|err| {
        //     eprintln!("{err}");
        //     err
        // });

        let framework = StandardFramework::new();
        let intents = merge_intents(&self.modules);

        // TODO: Use modules to initialize framework and intents.

        Ok(serenity::Client::builder(token, intents)
            .event_handler(self)
            .framework(framework)
            .await?
            .start()
            .await?)
    }

    fn create_application_commands(&self) -> Vec<CreateApplicationCommand> {
        self.slash_commands
            .iter()
            .map(|command| {
                let mut application_command = CreateApplicationCommand::default();
                application_command
                    .name(command.name())
                    .description(command.description());
                command.create_application_command(&mut application_command);
                application_command
            })
            .collect()
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        let already_connected = self.already_connected.swap(true, atomic::Ordering::AcqRel);
        let connected = if already_connected {
            "Reconnected"
        } else {
            "Connected"
        };
        let bot_name = data_about_bot.user.name;
        let guild_count = data_about_bot.guilds.len();
        let s = if guild_count == 1 { "" } else { "s" };
        println!("{connected} as {bot_name} to {guild_count} guild{s}.",);

        if already_connected {
            return;
        }

        let application_commands = self.create_application_commands();

        // TODO: Update commands depending on modules.
        async fn update_guilds(
            x: Vec<CreateApplicationCommand>,
            http: &Http,
            guilds: &Vec<GuildId>,
        ) -> Vec<(String, Option<serenity::Error>)> {
            let guild_names = join_all(guilds.iter().map(|guild| async move {
                format!(
                    "for {}",
                    &Guild::get(http, guild)
                        .await
                        .map(|guild| guild.name)
                        .unwrap_or_else(|_| format!("<{}>", guild.0))
                )
            }));
            let guild_updates = join_all(guilds.iter().map(|guild| async {
                guild
                    .set_application_commands(http, |commands| {
                        commands.set_application_commands(x.clone())
                    })
                    .await
                    .err()
            }));
            let (guild_names, guild_updates) = join!(guild_names, guild_updates);
            guild_names
                .into_iter()
                .zip(guild_updates.into_iter())
                .collect()
        }

        match &self.application_command_update {
            Some(ApplicationCommandUpdate::Global) => {
                println!("Updating application commands globally...");
                vec![(
                    "globally".to_owned(),
                    Command::set_global_application_commands(&ctx.http, |commands| {
                        commands.set_application_commands(application_commands)
                    })
                    .await
                    .err(),
                )]
            }
            Some(ApplicationCommandUpdate::Connected) => {
                println!(
                    "Updating application commands for all {guild_count} connected guild{s}..."
                );
                update_guilds(
                    application_commands,
                    &ctx.http,
                    &data_about_bot.guilds.iter().map(|guild| guild.id).collect(),
                )
                .await
            }
            Some(ApplicationCommandUpdate::Only(guilds)) => {
                let guild_count = guilds.len();
                let s = if guild_count == 1 { "" } else { "s" };
                println!("Updating application commands only for the {guild_count} specified guild{s}...");
                update_guilds(application_commands, &ctx.http, guilds).await
            }
            None => {
                println!("Skipping updating of application commands.");
                Vec::new()
            }
        }.into_iter().for_each(|(guild, error)| {
            match error {
                Some(error) => {
                    eprintln!("Error updating application commands {guild}:\n{error:#?}")
                }
                None => println!("Successfully updated application commands {guild}.",),
            }
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(interaction) => {
                let command_name = &interaction.data.name;
                let command = self
                    .slash_commands
                    .iter()
                    .find_map(|command| (command.name() == command_name).then_some(command));

                match command {
                    Some(command) => command.run(ctx, interaction).await,
                    None => {
                        interaction
                            .create_interaction_response(&ctx.http, |response| {
                                response.interaction_response_data(|data| {
                                    data.embed(|embed| {
                                        embed.color(colors::css::DANGER).field(
                                            format!(":x: Unknown command: `/{command_name}`"),
                                            "Bot commands are likely outdated.".to_string(),
                                            false,
                                        )
                                    })
                                    .ephemeral(true)
                                })
                            })
                            .await
                    }
                }
            }
            Interaction::Autocomplete(interaction) => {
                interaction
                    .create_autocomplete_response(&ctx.http, |response| response)
                    .await
            }
            _ => Ok(()),
        }
        .unwrap_or_else(|error| {
            eprintln!("Error creating interaction response:\n{error:#?}");
        });
    }
}

// #[async_trait]
// impl RawEventHandler for Bot {
//     async fn raw_event(&self, _ctx: Context, event: Event) {
//         println!("{event:#?}");
//     }
// }
