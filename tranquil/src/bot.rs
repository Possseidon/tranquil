use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use futures::{future::join_all, join};
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    framework::StandardFramework,
    http::Http,
    model::{
        application::{command::Command, interaction::Interaction},
        prelude::*,
    },
    prelude::*,
    utils::colours as colors,
};

use crate::{module::Module, slash_command::SlashCommands, AnyError, AnyResult};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplicationCommandUpdate {
    #[default]
    Global,
    Connected,
    Only(Vec<GuildId>),
}

pub struct Bot {
    already_connected: AtomicBool,
    application_command_update: Option<ApplicationCommandUpdate>,
    slash_commands: SlashCommands,
    modules: Vec<Arc<dyn Module>>,
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

    pub fn register(mut self, module: impl Module + 'static) -> Self {
        let module = Arc::new(module);
        self.slash_commands
            .append(&mut module.clone().slash_commands());
        self.modules.push(module);
        self
    }

    pub async fn run(self, token: impl AsRef<str>) -> AnyResult<()> {
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
                application_command.name(command.name()).description("TODO");
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
        let handle = async {
            match interaction {
                Interaction::ApplicationCommand(interaction) => {
                    let command_name = &interaction.data.name;
                    let command = self
                        .slash_commands
                        .iter()
                        .find(|command| command.name() == command_name);

                    match command {
                        Some(command) => command.run(ctx, interaction).await?,
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
                                .await?
                        }
                    }
                }
                Interaction::Autocomplete(interaction) => {
                    interaction
                        .create_autocomplete_response(&ctx.http, |response| response)
                        .await?
                }
                _ => {}
            };
            Ok::<(), AnyError>(())
        };
        handle.await.unwrap_or_else(|error| {
            eprintln!("Error creating interaction response:\n{error:#?}");
        });
    }
}

#[async_trait]
impl RawEventHandler for Bot {
    async fn raw_event(&self, _ctx: Context, event: Event) {
        println!("{event:#?}");
    }
}

fn merge_intents(modules: &Vec<Arc<dyn Module>>) -> GatewayIntents {
    modules.iter().fold(GatewayIntents::empty(), |acc, module| {
        acc | module.intents()
    })
}
