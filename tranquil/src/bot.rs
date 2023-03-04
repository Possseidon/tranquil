use std::{
    collections::{hash_map::Entry, HashMap},
    mem::take,
    ops::Deref,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use futures::{future::join_all, join};
use itertools::chain;
use serenity::{
    builder::CreateApplicationCommand,
    client::{EventHandler, RawEventHandler},
    http::Http,
    model::{
        application::{
            command::{Command, CommandOptionType},
            component::ComponentType,
            interaction::Interaction,
        },
        event::Event,
        gateway::{GatewayIntents, Ready},
        guild::{Guild, UnavailableGuild},
        id::GuildId,
    },
    utils::colours as colors,
    Client,
};
use uuid::Uuid;

use crate::{
    command::{CommandMap, CommandMapEntry, CommandPath, SubcommandMapEntry},
    context::{AutocompleteCtx, CommandCtx, Ctx, MessageComponentCtx, ModalCtx},
    l10n::{CommandPathRef, L10n},
    module::Module,
};

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplicationCommandUpdate {
    #[default]
    Global,
    Connected,
    Only(Vec<GuildId>),
}

type CustomIdMap = HashMap<Uuid, Arc<dyn Module>>;

pub struct Bot {
    already_connected: AtomicBool,
    application_command_update: Option<ApplicationCommandUpdate>,
    command_map: CommandMap,
    custom_id_map: CustomIdMap,
    modules: Vec<Arc<dyn Module>>,
    l10n: L10n,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            already_connected: Default::default(),
            application_command_update: Some(ApplicationCommandUpdate::default()),
            command_map: Default::default(),
            custom_id_map: Default::default(),
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

    pub fn register(mut self, module: impl Module + 'static) -> Self {
        self.modules.push(Arc::new(module));
        self
    }

    pub async fn run(mut self, discord_token: impl AsRef<str>) -> anyhow::Result<()> {
        // TODO: Token validation doesn't work, because of the middle "timestamp" part not always being valid base64.
        // validate_token(&token).map_err(|err| {
        //     eprintln!("{err}");
        //     err
        // });

        self.command_map = self.load_command_map()?;
        self.custom_id_map = self.load_custom_id_map()?;
        self.l10n = self.load_l10n().await?;

        let intents = merge_intents(self.modules.iter().map(Deref::deref));

        Client::builder(discord_token, intents)
            .event_handler(self)
            .await?
            .start()
            .await?;

        Ok(())
    }

    fn load_command_map(&self) -> anyhow::Result<CommandMap> {
        self.modules.iter().try_fold(
            Default::default(),
            |command_map, module| -> anyhow::Result<CommandMap> {
                Ok(command_map.merge(module.clone().command_map()?)?)
            },
        )
    }

    fn load_custom_id_map(&self) -> anyhow::Result<CustomIdMap> {
        let mut custom_id_map = CustomIdMap::new();
        for module in &self.modules {
            for &uuid in chain(module.interaction_uuids(), module.modal_uuids()) {
                match custom_id_map.entry(uuid) {
                    Entry::Vacant(entry) => entry.insert(module.clone()),
                    Entry::Occupied(_) => bail!("duplicate interaction uuid: {uuid}"),
                };
            }
        }
        Ok(custom_id_map)
    }

    async fn load_l10n(&self) -> anyhow::Result<L10n> {
        L10n::merge_results(join_all(self.modules.iter().map(|module| module.l10n())).await)
            .map_err(|error| {
                eprintln!("{error}");
                anyhow!("invalid l10n")
            })
    }

    pub async fn run_until_ctrl_c(self, discord_token: impl AsRef<str>) -> anyhow::Result<()> {
        tokio::select! {
            result = self.run(discord_token) => result?,
            result = tokio::signal::ctrl_c() => result?,
        }

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
        println!("{connected} as {bot_name} to {guild_count} guild{s}",);
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
            println!("Skipping updating of application commands");
        }
    }

    async fn handle_command(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        let command_path = CommandPath::resolve(&ctx.interaction.data);

        match self.command_map.find_command(&command_path) {
            Some(command) => command.run(ctx).await?,
            None => {
                ctx.create_interaction_response(|response| {
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
                .await?
            }
        }

        Ok(())
    }

    fn parse_custom_id<'custom_id>(
        &self,
        custom_id: &'custom_id str,
    ) -> anyhow::Result<(Uuid, &'custom_id str)> {
        custom_id
            .split_once(' ')
            .ok_or_else(|| anyhow!("invalid custom_id: {custom_id}"))
            .and_then(|(uuid, state)| {
                Ok((
                    Uuid::parse_str(uuid)
                        .with_context(|| format!("invalid custom_id uuid: {uuid}"))?,
                    state,
                ))
            })
    }

    async fn handle_message_component(&self, mut ctx: MessageComponentCtx) -> anyhow::Result<()> {
        match ctx.interaction.data.component_type {
            ComponentType::Button | ComponentType::SelectMenu => {
                let custom_id = take(&mut ctx.interaction.data.custom_id);
                let (uuid, state) = self.parse_custom_id(&custom_id)?;
                self.resolve_custom_id_module(uuid)?
                    .interact(uuid, state, ctx)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_autocomplete(&self, ctx: AutocompleteCtx) -> anyhow::Result<()> {
        let command_path = CommandPath::resolve(&ctx.interaction.data);

        match self.command_map.find_command(&command_path) {
            Some(command) => command.autocomplete(ctx).await?,
            None => {
                // Commands are probably outdated... Send an empty autocomplete response.
                ctx.create_autocomplete_response(|response| response)
                    .await?
            }
        }

        Ok(())
    }

    async fn handle_modal(&self, mut ctx: ModalCtx) -> anyhow::Result<()> {
        let custom_id = take(&mut ctx.interaction.data.custom_id);
        let (uuid, state) = self.parse_custom_id(&custom_id)?;
        self.resolve_custom_id_module(uuid)?
            .submit(uuid, state, ctx)
            .await
    }

    fn resolve_custom_id_module(&self, uuid: Uuid) -> anyhow::Result<&Arc<dyn Module>> {
        self.custom_id_map
            .get(&uuid)
            .ok_or_else(|| anyhow!("no module that handles the custom_id uuid {uuid}"))
    }
}

type GuildUpdateError = (String, Result<(), serenity::Error>);

fn print_application_command_update_errors(
    guild_update_errors: impl Iterator<Item = GuildUpdateError>,
) {
    for (guild, error) in guild_update_errors {
        if let Err(error) = error {
            eprintln!(" ⚠ {guild}\n   ▶ {error}");
        } else {
            println!(" ✓ {guild}");
        }
    }
}

async fn update_application_commands_globally(
    http: &Http,
    application_commands: Vec<CreateApplicationCommand>,
) {
    let command_count = application_commands.len();
    println!(
        "Updating {command_count} application command{} globally...",
        if command_count == 1 { "" } else { "s" }
    );
    print_application_command_update_errors(
        [(
            "globally".to_owned(),
            Command::set_global_application_commands(http, |commands| {
                commands.set_application_commands(application_commands)
            })
            .await
            .map(|_| ()),
        )]
        .into_iter(),
    );
}

async fn update_application_commands_for_connected_guilds(
    http: &Http,
    application_commands: Vec<CreateApplicationCommand>,
    connected_guilds: &[UnavailableGuild],
) {
    let command_count = application_commands.len();
    let guild_count = connected_guilds.len();
    println!(
        "Updating {command_count} application command{} for all {guild_count} connected guild{}...",
        if command_count == 1 { "" } else { "s" },
        if guild_count == 1 { "" } else { "s" },
    );
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
    let command_count = application_commands.len();
    let guild_count = guilds.len();
    println!(
        "Updating {command_count} application command{} only for the {guild_count} specified guild{}...",
        if command_count == 1 { "" } else { "s" },
        if guild_count == 1 { "" } else { "s" },
    );
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
        Guild::get(http, guild)
            .await
            .map(|guild| guild.name)
            .unwrap_or_else(|_| format!("<{}>", guild.0))
    }));
    let guild_updates = join_all(guilds.clone().map(|guild| {
        let create_application_commands = create_application_commands.clone();
        async move {
            guild
                .set_application_commands(http, |commands| {
                    commands.set_application_commands(create_application_commands)
                })
                .await
                .map(|_| ())
        }
    }));
    let (guild_names, guild_updates) = join!(guild_names, guild_updates);
    guild_names.into_iter().zip(guild_updates.into_iter())
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, bot: serenity::client::Context, data_about_bot: Ready) {
        if self.notify_connect(&data_about_bot.user.name, data_about_bot.guilds.len()) {
            self.update_application_commands(&bot.http, &data_about_bot.guilds)
                .await;
        }
        println!("Ready!");
        println!();
    }

    async fn interaction_create(&self, bot: serenity::client::Context, interaction: Interaction) {
        async {
            match interaction {
                Interaction::Ping(_) => {}
                Interaction::ApplicationCommand(interaction) => {
                    self.handle_command(Ctx { bot, interaction }).await?
                }
                Interaction::MessageComponent(interaction) => {
                    self.handle_message_component(Ctx { bot, interaction })
                        .await?
                }
                Interaction::Autocomplete(interaction) => {
                    self.handle_autocomplete(Ctx { bot, interaction }).await?
                }
                Interaction::ModalSubmit(interaction) => {
                    self.handle_modal(Ctx { bot, interaction }).await?
                }
            }

            Ok(())
        }
        .await
        .unwrap_or_else(|error: anyhow::Error| {
            let error = error.context("error during interaction");
            eprintln!(" ⚠  {:?}", error);
            eprintln!();
        });
    }
}

#[async_trait]
impl RawEventHandler for Bot {
    async fn raw_event(&self, _bot: serenity::client::Context, event: Event) {
        println!("{event:#?}");
    }
}

fn merge_intents<'a>(modules: impl Iterator<Item = &'a dyn Module>) -> GatewayIntents {
    modules.fold(GatewayIntents::empty(), |acc, module| {
        acc | module.intents()
    })
}
