use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
};

use futures::Future;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    client::Context,
    model::application::{
        command::CommandOptionType,
        interaction::application_command::{
            ApplicationCommandInteraction, CommandData, CommandDataOption,
        },
    },
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

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPath {
    Command {
        name: String,
    },
    Subcommand {
        name: String,
        subcommand: String,
    },
    Grouped {
        name: String,
        group: String,
        subcommand: String,
    },
}

impl CommandPath {
    pub(crate) fn name(&self) -> &str {
        match self {
            CommandPath::Command { name }
            | CommandPath::Subcommand { name, .. }
            | CommandPath::Grouped { name, .. } => name,
        }
    }
}

#[derive(Debug)]
pub struct InvalidCommandData;

impl Display for InvalidCommandData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid command data")
    }
}

impl Error for InvalidCommandData {}

pub(crate) fn resolve_command_path(command_data: &CommandData) -> CommandPath {
    match command_data.options.as_slice() {
        [group]
            if group.kind == CommandOptionType::SubCommand
                || group.kind == CommandOptionType::SubCommandGroup =>
        {
            match group.options.as_slice() {
                [subcommand] if subcommand.kind == CommandOptionType::SubCommand => {
                    CommandPath::Grouped {
                        name: command_data.name.clone(),
                        group: group.name.clone(),
                        subcommand: subcommand.name.clone(),
                    }
                }
                _ => CommandPath::Subcommand {
                    name: command_data.name.clone(),
                    subcommand: group.name.clone(),
                },
            }
        }
        _ => CommandPath::Command {
            name: command_data.name.clone(),
        },
    }
}

pub fn resolve_command_options(command_data: &CommandData) -> &[CommandDataOption] {
    match command_data.options.as_slice() {
        [group]
            if group.kind == CommandOptionType::SubCommand
                || group.kind == CommandOptionType::SubCommandGroup =>
        {
            match group.options.as_slice() {
                [subcommand] if subcommand.kind == CommandOptionType::SubCommand => {
                    &subcommand.options
                }
                _ => &group.options,
            }
        }
        _ => &command_data.options,
    }
}

impl Display for CommandPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandPath::Command { name } => write!(f, "{name}"),
            CommandPath::Subcommand { name, subcommand } => write!(f, "{name} {subcommand}"),
            CommandPath::Grouped {
                name,
                group,
                subcommand,
            } => write!(f, "{name} {group} {subcommand}"),
        }
    }
}

pub type OptionBuilder = fn(&TranslatedCommands) -> CreateApplicationCommandOption;

pub struct ModuleCommand<M: Module> {
    function: CommandFunction<M>,
    option_builders: Vec<OptionBuilder>,
    module: Arc<M>,
}

impl<M: Module> ModuleCommand<M> {
    pub fn new(
        function: CommandFunction<M>,
        option_builders: Vec<OptionBuilder>,
        module: Arc<M>,
    ) -> Self {
        Self {
            function,
            option_builders,
            module,
        }
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    fn add_options(
        &self,
        translated_commands: &TranslatedCommands,
        command: &mut CreateApplicationCommand,
    );

    fn add_suboptions(
        &self,
        translated_commands: &TranslatedCommands,
        option: &mut CreateApplicationCommandOption,
    );

    async fn run(&self, ctx: Context, interaction: ApplicationCommandInteraction) -> AnyResult<()>;
}

#[async_trait]
impl<M: Module> Command for ModuleCommand<M> {
    fn add_options(
        &self,
        translated_commands: &TranslatedCommands,
        command: &mut CreateApplicationCommand,
    ) {
        for option_builder in &self.option_builders {
            command.add_option(option_builder(translated_commands));
        }
    }

    fn add_suboptions(
        &self,
        translated_commands: &TranslatedCommands,
        command: &mut CreateApplicationCommandOption,
    ) {
        for option_builder in &self.option_builders {
            command.add_sub_option(option_builder(translated_commands));
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

#[derive(Debug)]
pub enum CommandMapMergeError {
    DuplicateCommand { path: CommandPath },
    AmbiguousSubcommand { path: CommandPath },
}

impl Error for CommandMapMergeError {}

impl Display for CommandMapMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandMapMergeError::DuplicateCommand { path } => {
                write!(f, "duplicate command `/{path}`")
            }
            CommandMapMergeError::AmbiguousSubcommand { path } => {
                write!(f, "command `/{path}` cannot also have subcommands")
            }
        }
    }
}

pub struct CommandContext {
    pub ctx: Context,
    pub interaction: ApplicationCommandInteraction,
}

pub enum SubcommandMapEntry {
    Subcommand(Box<dyn Command>),
    Group(HashMap<String, Box<dyn Command>>),
}

pub type SubcommandMap = HashMap<String, SubcommandMapEntry>;

pub enum CommandMapEntry {
    Command(Box<dyn Command>),
    Subcommands(SubcommandMap),
}

pub type CommandMap = HashMap<String, CommandMapEntry>;

fn command_map_entry(path: CommandPath, command: Box<dyn Command>) -> (String, CommandMapEntry) {
    match path {
        CommandPath::Command { name } => (name, CommandMapEntry::Command(command)),
        CommandPath::Subcommand { name, subcommand } => (
            name,
            CommandMapEntry::Subcommands(
                [(subcommand, SubcommandMapEntry::Subcommand(command))].into(),
            ),
        ),
        CommandPath::Grouped {
            name,
            group,
            subcommand,
        } => (
            name,
            CommandMapEntry::Subcommands(
                [(
                    group,
                    SubcommandMapEntry::Group([(subcommand, command)].into()),
                )]
                .into(),
            ),
        ),
    }
}

pub fn create_command_map(
    commands: impl IntoIterator<Item = (CommandPath, Box<dyn Command>)>,
) -> Result<CommandMap, CommandMapMergeError> {
    merge_command_maps(
        Default::default(),
        commands
            .into_iter()
            .map(|(path, command)| command_map_entry(path, command)),
    )
}

pub(crate) fn merge_command_maps(
    mut command_map: CommandMap,
    new_commands: impl IntoIterator<Item = (String, CommandMapEntry)>,
) -> Result<CommandMap, CommandMapMergeError> {
    for (name, new_entry) in new_commands {
        command_map_add_entry(&mut command_map, name, new_entry)?;
    }
    Ok(command_map)
}

fn merge_subcommand_groups(
    group_commandmap: &mut HashMap<String, Box<dyn Command>>,
    name: String,
    group: String,
    new_group_commandmap: HashMap<String, Box<dyn Command>>,
) -> Result<(), CommandMapMergeError> {
    for (subcommand, new_entry) in new_group_commandmap {
        if let Entry::Vacant(entry) = group_commandmap.entry(subcommand.clone()) {
            entry.insert(new_entry);
        } else {
            Err(CommandMapMergeError::DuplicateCommand {
                path: CommandPath::Grouped {
                    name: name.clone(),
                    group: group.clone(),
                    subcommand,
                },
            })?
        }
    }
    Ok(())
}

fn merge_subcommand_map_entry(
    entry: &mut SubcommandMapEntry,
    name: String,
    subcommand: String,
    new_entry: SubcommandMapEntry,
) -> Result<(), CommandMapMergeError> {
    match (entry, new_entry) {
        (SubcommandMapEntry::Subcommand(_), SubcommandMapEntry::Subcommand(_)) => {
            Err(CommandMapMergeError::DuplicateCommand {
                path: CommandPath::Subcommand { name, subcommand },
            })
        }
        (SubcommandMapEntry::Subcommand(_), SubcommandMapEntry::Group(_))
        | (SubcommandMapEntry::Group(_), SubcommandMapEntry::Subcommand(_)) => {
            Err(CommandMapMergeError::AmbiguousSubcommand {
                path: CommandPath::Subcommand { name, subcommand },
            })
        }
        (SubcommandMapEntry::Group(group), SubcommandMapEntry::Group(new_group)) => {
            merge_subcommand_groups(group, name, subcommand, new_group)
        }
    }
}

fn merge_command_map_entry(
    entry: &mut CommandMapEntry,
    name: String,
    new_entry: CommandMapEntry,
) -> Result<(), CommandMapMergeError> {
    match (entry, new_entry) {
        (CommandMapEntry::Command(_), CommandMapEntry::Command(_)) => {
            Err(CommandMapMergeError::DuplicateCommand {
                path: CommandPath::Command { name },
            })?
        }
        (CommandMapEntry::Command(_), CommandMapEntry::Subcommands(_))
        | (CommandMapEntry::Subcommands(_), CommandMapEntry::Command(_)) => {
            Err(CommandMapMergeError::AmbiguousSubcommand {
                path: CommandPath::Command { name },
            })?
        }
        (
            CommandMapEntry::Subcommands(subcommand_map),
            CommandMapEntry::Subcommands(new_subcommands),
        ) => {
            for (subcommand, new_entry) in new_subcommands {
                match subcommand_map.entry(subcommand.clone()) {
                    Entry::Occupied(mut entry) => merge_subcommand_map_entry(
                        entry.get_mut(),
                        name.clone(),
                        subcommand,
                        new_entry,
                    )?,
                    Entry::Vacant(entry) => {
                        entry.insert(new_entry);
                    }
                }
            }
        }
    }
    Ok(())
}

fn command_map_add_entry(
    command_map: &mut HashMap<String, CommandMapEntry>,
    name: String,
    new_entry: CommandMapEntry,
) -> Result<(), CommandMapMergeError> {
    match command_map.entry(name.clone()) {
        Entry::Occupied(mut entry) => merge_command_map_entry(entry.get_mut(), name, new_entry)?,
        Entry::Vacant(entry) => {
            entry.insert(new_entry);
        }
    }
    Ok(())
}

pub(crate) fn find_command<'a>(
    command_map: &'a CommandMap,
    command_path: &CommandPath,
) -> Option<&'a dyn Command> {
    command_map
        .get(command_path.name())
        .and_then(|entry| match (&command_path, entry) {
            (CommandPath::Command { .. }, CommandMapEntry::Command(command)) => Some(command),
            (
                CommandPath::Subcommand { subcommand, .. }
                | CommandPath::Grouped {
                    group: subcommand, ..
                },
                CommandMapEntry::Subcommands(subcommand_map),
            ) => subcommand_map
                .get(subcommand)
                .and_then(|entry| match (&command_path, entry) {
                    (
                        CommandPath::Subcommand { .. },
                        SubcommandMapEntry::Subcommand(subcommand),
                    ) => Some(subcommand),
                    (
                        CommandPath::Grouped { subcommand, .. },
                        SubcommandMapEntry::Group(subcommands),
                    ) => subcommands.get(subcommand),
                    _ => None,
                }),
            _ => None,
        })
        .map(|command| command.as_ref())
}

pub trait CommandProvider {
    fn command_map(self: Arc<Self>) -> Result<CommandMap, CommandMapMergeError>;
}
