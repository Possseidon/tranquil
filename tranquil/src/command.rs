use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
};

use anyhow::bail;
use async_trait::async_trait;
use futures::Future;
use serenity::{
    builder::{
        CreateApplicationCommand, CreateApplicationCommandOption, CreateInteractionResponseFollowup,
    },
    model::application::{
        command::CommandOptionType, interaction::application_command::CommandData,
    },
};
use thiserror::Error;

use crate::{
    autocomplete::AutocompleteFunction,
    context::{AutocompleteCtx, CommandCtx},
    l10n::L10n,
    module::Module,
};

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

    pub(crate) fn resolve(command_data: &CommandData) -> CommandPath {
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

type CommandFunction<M> = Box<
    dyn Fn(Arc<M>, CommandCtx) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
        + Send
        + Sync,
>;

pub type OptionBuilder = fn(&L10n) -> CreateApplicationCommandOption;

pub struct ModuleCommand<M: Module> {
    module: Arc<M>,
    command_function: CommandFunction<M>,
    autocomplete_function: Option<AutocompleteFunction<M>>,
    options: Vec<(String, OptionBuilder)>,
    default_option: bool,
}

impl<M: Module> ModuleCommand<M> {
    pub fn new(
        module: Arc<M>,
        command_function: CommandFunction<M>,
        autocomplete_function: Option<AutocompleteFunction<M>>,
        options: Vec<(String, OptionBuilder)>,
        default_option: bool,
    ) -> Self {
        Self {
            module,
            command_function,
            autocomplete_function,
            options,
            default_option,
        }
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    fn is_default_option(&self) -> bool;

    fn options(&self) -> Vec<String>;

    fn add_options(&self, l10n: &L10n, command: &mut CreateApplicationCommand);
    fn add_suboptions(&self, l10n: &L10n, option: &mut CreateApplicationCommandOption);

    async fn run(&self, ctx: CommandCtx) -> anyhow::Result<()>;
    async fn autocomplete(&self, ctx: AutocompleteCtx) -> anyhow::Result<()>;
}

impl Debug for dyn Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn Command")
    }
}

#[async_trait]
impl<M: Module> Command for ModuleCommand<M> {
    fn is_default_option(&self) -> bool {
        self.default_option
    }

    fn options(&self) -> Vec<String> {
        self.options.iter().map(|(name, _)| name.clone()).collect()
    }

    fn add_options(&self, l10n: &L10n, command: &mut CreateApplicationCommand) {
        for (_, option_builder) in &self.options {
            command.add_option(option_builder(l10n));
        }
    }

    fn add_suboptions(&self, l10n: &L10n, command: &mut CreateApplicationCommandOption) {
        for (_, option_builder) in &self.options {
            command.add_sub_option(option_builder(l10n));
        }
    }

    async fn run(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        (self.command_function)(self.module.clone(), ctx).await
    }

    async fn autocomplete(&self, ctx: AutocompleteCtx) -> anyhow::Result<()> {
        if let Some(autocomplete_function) = &self.autocomplete_function {
            autocomplete_function(self.module.clone(), ctx).await
        } else {
            bail!("no autocomplete handler")
        }
    }
}

pub(crate) struct CommandError(Box<dyn CommandErrorReporter>);

pub trait CommandErrorReporter: std::error::Error + Send + Sync {
    fn write_log(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn log_to_console(&self) -> bool;
    fn build_response(&self, response: &mut CreateInteractionResponseFollowup);
}

impl Debug for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CommandError").field(&self.0).finish()
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_log(f)
    }
}

impl std::error::Error for CommandError {}

impl CommandErrorReporter for CommandError {
    fn write_log(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_log(f)
    }

    fn log_to_console(&self) -> bool {
        self.0.log_to_console()
    }

    fn build_response(&self, response: &mut CreateInteractionResponseFollowup) {
        self.0.build_response(response)
    }
}

#[derive(Debug, Error)]
pub enum CommandMapMergeError {
    #[error("duplicate command `/{path}`")]
    DuplicateCommand { path: CommandPath },
    #[error("command `/{path}` cannot also have subcommands")]
    AmbiguousSubcommand { path: CommandPath },
}

#[derive(Debug, Default)]
pub struct CommandMap(HashMap<String, CommandMapEntry>);

#[derive(Debug)]
pub enum CommandMapEntry {
    Command(Box<dyn Command>),
    Subcommands(SubcommandMap),
}

#[derive(Debug, Default)]
pub struct SubcommandMap(HashMap<String, SubcommandMapEntry>);

#[derive(Debug)]
pub enum SubcommandMapEntry {
    Subcommand(Box<dyn Command>),
    Group(SubcommandGroupMap),
}

#[derive(Debug, Default)]
pub struct SubcommandGroupMap(HashMap<String, Box<dyn Command>>);

impl CommandMap {
    pub fn new(
        commands: impl IntoIterator<Item = (CommandPath, Box<dyn Command>)>,
    ) -> Result<Self, CommandMapMergeError> {
        Self::default().merge(
            commands
                .into_iter()
                .map(|(path, command)| CommandMapEntry::name_and_new(path, command)),
        )
    }

    pub(crate) fn merge(
        mut self,
        commands: impl IntoIterator<Item = (String, CommandMapEntry)>,
    ) -> Result<Self, CommandMapMergeError> {
        for (name, new_entry) in commands {
            self.add_entry(name, new_entry)?;
        }
        Ok(self)
    }

    fn add_entry(
        &mut self,
        name: String,
        new_entry: CommandMapEntry,
    ) -> Result<(), CommandMapMergeError> {
        match self.0.entry(name.clone()) {
            Entry::Occupied(mut entry) => entry.get_mut().merge(name, new_entry)?,
            Entry::Vacant(entry) => {
                entry.insert(new_entry);
            }
        }
        Ok(())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &CommandMapEntry)> {
        self.0.iter()
    }

    pub(crate) fn find_command<'a>(
        &'a self,
        command_path: &CommandPath,
    ) -> Option<&'a dyn Command> {
        self.0
            .get(command_path.name())
            .and_then(|entry| match (&command_path, entry) {
                (CommandPath::Command { .. }, CommandMapEntry::Command(command)) => Some(command),
                (
                    CommandPath::Subcommand { subcommand, .. }
                    | CommandPath::Grouped {
                        group: subcommand, ..
                    },
                    CommandMapEntry::Subcommands(subcommand_map),
                ) => subcommand_map.0.get(subcommand).and_then(|entry| {
                    match (&command_path, entry) {
                        (
                            CommandPath::Subcommand { .. },
                            SubcommandMapEntry::Subcommand(subcommand),
                        ) => Some(subcommand),
                        (
                            CommandPath::Grouped { subcommand, .. },
                            SubcommandMapEntry::Group(subcommands),
                        ) => subcommands.0.get(subcommand),
                        _ => None,
                    }
                }),
                _ => None,
            })
            .map(|command| command.as_ref())
    }
}

impl IntoIterator for CommandMap {
    type Item = (String, CommandMapEntry);
    type IntoIter = <HashMap<String, CommandMapEntry> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl CommandMapEntry {
    fn name_and_new(path: CommandPath, command: Box<dyn Command>) -> (String, Self) {
        match path {
            CommandPath::Command { name } => (name, Self::Command(command)),
            CommandPath::Subcommand { name, subcommand } => (
                name,
                Self::Subcommands(SubcommandMap(
                    [(subcommand, SubcommandMapEntry::Subcommand(command))].into(),
                )),
            ),
            CommandPath::Grouped {
                name,
                group,
                subcommand,
            } => (
                name,
                Self::Subcommands(SubcommandMap(
                    [(
                        group,
                        SubcommandMapEntry::Group(SubcommandGroupMap(
                            [(subcommand, command)].into(),
                        )),
                    )]
                    .into(),
                )),
            ),
        }
    }

    fn merge(
        &mut self,
        name: String,
        new_entry: CommandMapEntry,
    ) -> Result<(), CommandMapMergeError> {
        match (self, new_entry) {
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
                for (subcommand, new_entry) in new_subcommands.0 {
                    match subcommand_map.0.entry(subcommand.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().merge(name.clone(), subcommand, new_entry)?
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(new_entry);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a SubcommandMap {
    type Item = (&'a String, &'a SubcommandMapEntry);
    type IntoIter = <&'a HashMap<String, SubcommandMapEntry> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl SubcommandMapEntry {
    fn merge(
        &mut self,
        name: String,
        subcommand: String,
        new_entry: SubcommandMapEntry,
    ) -> Result<(), CommandMapMergeError> {
        match (self, new_entry) {
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
                group.merge(name, subcommand, new_group)
            }
        }
    }
}

impl SubcommandGroupMap {
    fn merge(
        &mut self,
        name: String,
        group: String,
        new_group_commandmap: SubcommandGroupMap,
    ) -> Result<(), CommandMapMergeError> {
        for (subcommand, new_entry) in new_group_commandmap.0 {
            if let Entry::Vacant(entry) = self.0.entry(subcommand.clone()) {
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
}

impl<'a> IntoIterator for &'a SubcommandGroupMap {
    type Item = (&'a String, &'a Box<dyn Command>);
    type IntoIter = <&'a HashMap<String, Box<dyn Command>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub trait CommandProvider {
    fn command_map(self: Arc<Self>) -> Result<CommandMap, CommandMapMergeError>;
}
