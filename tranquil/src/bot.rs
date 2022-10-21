use std::{
    ops::Deref,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use futures::{future::join_all, join};
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::{Context, EventHandler, RawEventHandler},
    framework::StandardFramework,
    http::Http,
    model::{
        application::{
            command::{Command, CommandOptionType},
            interaction::Interaction,
        },
        event::Event,
        gateway::{GatewayIntents, Ready},
        guild::{Guild, UnavailableGuild},
        id::GuildId,
    },
    utils::colours as colors,
};

use crate::{
    autocomplete::AutocompleteContext,
    command::{
        CommandContext, CommandMap, CommandMapEntry, CommandMapMergeError, CommandPath,
        SubcommandMapEntry,
    },
    l10n::{CommandPathRef, L10n},
    module::Module,
    AnyError, AnyResult,
};

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplicationCommandUpdate {
    #[default]
    Global,
    Connected,
    Only(Vec<GuildId>),
}

pub struct Bot {
    already_connected: AtomicBool,
    application_command_update: Option<ApplicationCommandUpdate>,
    command_map: CommandMap,
    modules: Vec<Arc<dyn Module>>,
    l10n: L10n,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            already_connected: Default::default(),
            application_command_update: Some(ApplicationCommandUpdate::default()),
            command_map: Default::default(),
            modules: Default::default(),
            l10n: Default::default(),
        }
    }
}

impl Bot {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn application_command_update(
        mut self,
        application_command_update: Option<ApplicationCommandUpdate>,
    ) -> Self {
        self.application_command_update = application_command_update;
        self
    }

    pub fn register(mut self, module: impl Module + 'static) -> Result<Self, CommandMapMergeError> {
        let module = Arc::new(module);
        self.command_map = self.command_map.merge(module.clone().command_map()?)?;
        self.modules.push(module);
        Ok(self)
    }

    pub async fn run(mut self, token: impl AsRef<str>) -> AnyResult<()> {
        // TODO: Token validation doesn't work, because of the middle "timestamp" part not always being valid base64.
        // validate_token(&token).map_err(|err| {
        //     eprintln!("{err}");
        //     err
        // });

        self.load_translations().await?;

        let framework = StandardFramework::new();
        let intents = merge_intents(self.modules.iter().map(Deref::deref));

        // TODO: Use modules to initialize framework and intents.

        Ok(serenity::Client::builder(token, intents)
            .event_handler(self)
            .framework(framework)
            .await?
            .start()
            .await?)
    }

    async fn load_translations(&mut self) -> AnyResult<()> {
        self.l10n =
            L10n::from_yaml_files(self.modules.iter().filter_map(|module| module.l10n_path()))
                .await?;
        Ok(())
    }

    fn create_application_commands(&self) -> Vec<CreateApplicationCommand> {
        self.command_map
            .iter()
            .map(|(name, command)| {
                let mut application_command = CreateApplicationCommand::default();

                self.l10n.describe_command(name, &mut application_command);

                match command {
                    CommandMapEntry::Command(command) => {
                        command.add_options(&self.l10n, &mut application_command);
                    }
                    CommandMapEntry::Subcommands(subcommands) => {
                        for (subcommand, entry) in subcommands {
                            application_command.create_option(|option| {
                                self.l10n.describe_subcommand(
                                    CommandPathRef::Subcommand { name, subcommand },
                                    option,
                                );

                                match entry {
                                    SubcommandMapEntry::Subcommand(command) => {
                                        option
                                            .kind(CommandOptionType::SubCommand)
                                            .default_option(command.is_default_option());

                                        command.add_suboptions(&self.l10n, option);
                                    }
                                    SubcommandMapEntry::Group(command_map) => {
                                        let group = subcommand;
                                        option.kind(CommandOptionType::SubCommandGroup);
                                        for (subcommand, command) in command_map {
                                            option.create_sub_option(|option| {
                                                self.l10n.describe_subcommand(
                                                    CommandPathRef::Grouped {
                                                        name,
                                                        group,
                                                        subcommand,
                                                    },
                                                    option,
                                                );

                                                option
                                                    .kind(CommandOptionType::SubCommand)
                                                    .default_option(command.is_default_option());

                                                command.add_suboptions(&self.l10n, option);

                                                option
                                            });
                                        }
                                    }
                                }

                                option
                            });
                        }
                    }
                }

                application_command
            })
            .collect()
    }

    fn notify_connect(&self, bot_name: &str, guild_count: usize) -> bool {
        let first_connect = !self.already_connected.swap(true, atomic::Ordering::AcqRel);
        let connected = if first_connect {
            "Connected"
        } else {
            "Reconnected"
        };
        let s = if guild_count == 1 { "" } else { "s" };
        println!("{connected} as {bot_name} to {guild_count} guild{s}.",);
        first_connect
    }

    async fn update_application_commands(&self, http: &Http, global_guilds: &[UnavailableGuild]) {
        if let Some(application_command_update) = &self.application_command_update {
            update_application_commands(
                application_command_update,
                self.create_application_commands(),
                http,
                global_guilds,
            )
            .await;
        } else {
            println!("Skipping updating of application commands.");
        }
    }
}

type GuildUpdateError = (String, Option<serenity::Error>);

fn print_application_command_update_errors(
    guild_update_errors: impl Iterator<Item = GuildUpdateError>,
) {
    for (for_guild, error) in guild_update_errors {
        if let Some(error) = error {
            eprintln!("Error updating application commands {for_guild}:\n{error}")
        } else {
            println!("Successfully updated application commands {for_guild}.",)
        }
    }
}

async fn update_application_commands_globally(
    http: &Http,
    application_commands: Vec<CreateApplicationCommand>,
) {
    println!("Updating application commands globally...");
    print_application_command_update_errors(
        [(
            "globally".to_owned(),
            Command::set_global_application_commands(http, |commands| {
                commands.set_application_commands(application_commands)
            })
            .await
            .err(),
        )]
        .into_iter(),
    );
}

async fn update_application_commands_for_connected_guilds(
    http: &Http,
    application_commands: Vec<CreateApplicationCommand>,
    connected_guilds: &[UnavailableGuild],
) {
    let guild_count = connected_guilds.len();
    let s = if guild_count == 1 { "" } else { "s" };
    println!("Updating application commands for all {guild_count} connected guild{s}...");
    print_application_command_update_errors(
        update_guilds(
            application_commands,
            http,
            connected_guilds.iter().map(|guild| guild.id),
        )
        .await,
    );
}

async fn update_application_commands_for(
    http: &Http,
    application_commands: Vec<CreateApplicationCommand>,
    guilds: &[GuildId],
) {
    let guild_count = guilds.len();
    let s = if guild_count == 1 { "" } else { "s" };
    println!("Updating application commands only for the {guild_count} specified guild{s}...");
    print_application_command_update_errors(
        update_guilds(application_commands, http, guilds.iter().copied()).await,
    );
}

async fn update_application_commands(
    application_command_update: &ApplicationCommandUpdate,
    application_commands: Vec<CreateApplicationCommand>,
    http: &Http,
    connected_guilds: &[UnavailableGuild],
) {
    match application_command_update {
        ApplicationCommandUpdate::Global => {
            update_application_commands_globally(http, application_commands).await
        }
        ApplicationCommandUpdate::Connected => {
            update_application_commands_for_connected_guilds(
                http,
                application_commands,
                connected_guilds,
            )
            .await
        }
        ApplicationCommandUpdate::Only(guilds) => {
            update_application_commands_for(http, application_commands, guilds).await
        }
    }
}

async fn update_guilds(
    create_application_commands: impl Into<Vec<CreateApplicationCommand>>,
    http: &Http,
    guilds: impl Iterator<Item = GuildId> + Clone,
) -> impl Iterator<Item = GuildUpdateError> {
    let create_application_commands = create_application_commands.into();
    let guild_names = join_all(guilds.clone().map(|guild| async move {
        format!(
            "for {}",
            &Guild::get(http, guild)
                .await
                .map(|guild| guild.name)
                .unwrap_or_else(|_| format!("<{}>", guild.0))
        )
    }));
    let guild_updates = join_all(guilds.clone().map(|guild| {
        let create_application_commands = create_application_commands.clone();
        async move {
            guild
                .set_application_commands(http, |commands| {
                    commands.set_application_commands(create_application_commands)
                })
                .await
                .err()
        }
    }));
    let (guild_names, guild_updates) = join!(guild_names, guild_updates);
    guild_names.into_iter().zip(guild_updates.into_iter())
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, bot: Context, data_about_bot: Ready) {
        if self.notify_connect(&data_about_bot.user.name, data_about_bot.guilds.len()) {
            self.update_application_commands(&bot.http, &data_about_bot.guilds)
                .await;
        }
    }

    async fn interaction_create(&self, bot: Context, interaction: Interaction) {
        async {
            match interaction {
                Interaction::ApplicationCommand(interaction) => {
                    let command_path = CommandPath::resolve(&interaction.data);
                    match self.command_map.find_command(&command_path) {
                        Some(command) => command.run(CommandContext { bot, interaction }).await?,
                        None => {
                            interaction
                                .create_interaction_response(bot, |response| {
                                    response.interaction_response_data(|data| {
                                        data.embed(|embed| {
                                            embed.color(colors::css::DANGER).field(
                                                format!(":x: Unknown command: `/{command_path}`"),
                                                "Bot commands are likely outdated.".to_string(),
                                                false,
                                            )
                                        })
                                        .ephemeral(true)
                                    })
                                })
                                .await?;
                        }
                    }
                }
                Interaction::Autocomplete(interaction) => {
                    let command_path = CommandPath::resolve(&interaction.data);
                    match self.command_map.find_command(&command_path) {
                        Some(command) => {
                            command
                                .autocomplete(AutocompleteContext { bot, interaction })
                                .await?;
                        }
                        None => {
                            // Commands are probably outdated... Send an empty autocomplete response.
                            interaction
                                .create_autocomplete_response(bot, |response| response)
                                .await?;
                        }
                    }
                }
                _ => {}
            }
            Ok::<(), AnyError>(())
        }
        .await
        .unwrap_or_else(|error| {
            eprintln!("Error creating interaction response:\n{error}");
        });
    }
}

#[async_trait]
impl RawEventHandler for Bot {
    async fn raw_event(&self, _bot: Context, event: Event) {
        println!("{event:#?}");
    }
}

fn merge_intents<'a>(modules: impl Iterator<Item = &'a dyn Module>) -> GatewayIntents {
    modules.fold(GatewayIntents::empty(), |acc, module| {
        acc | module.intents()
    })
}
