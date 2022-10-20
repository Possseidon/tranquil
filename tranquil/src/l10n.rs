use std::{
    collections::{btree_map::Entry, BTreeMap},
    iter::zip,
};

use enumset::{EnumSet, EnumSetType};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};

use crate::{
    command::{Command, CommandMap, CommandMapEntry, SubcommandMapEntry},
    resolve::Choices,
};

pub trait CommandL10nProvider {
    fn l10n_path(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct L10n {
    #[serde(default)]
    commands: BTreeMap<String, CommandL10n>,
    #[serde(default)]
    choices: BTreeMap<String, ChoiceL10n>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CommandL10n {
    #[serde(default)]
    name: Translations,
    #[serde(default)]
    description: Translations,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    subcommands: BTreeMap<String, CommandL10n>,
    #[serde(default, with = "tuple_vec_map", skip_serializing_if = "Vec::is_empty")]
    options: Vec<(String, OptionL10n)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OptionL10n {
    #[serde(default)]
    name: Translations,
    #[serde(default)]
    description: Translations,
}

impl OptionL10n {
    fn stubs(locales: EnumSet<Locale>) -> Self {
        let mut l10n = Self::default();
        l10n.fill_stubs(locales);
        l10n
    }

    fn fill_stubs(&mut self, locales: EnumSet<Locale>) {
        self.name.fill_stubs(locales);
        self.description.fill_stubs(locales);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
struct ChoiceL10n(#[serde(with = "tuple_vec_map")] Vec<(String, Translations)>);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
struct Translations(BTreeMap<Locale, String>);

macro_rules! make_locale {
    {$($(#[$default:ident])? $name:ident = $code:literal),* $(,)?} => {
        #[derive(
            Debug, Default, PartialOrd, Ord, Deserialize, Serialize, EnumSetType
        )]
        #[serde(into = "&str", try_from = "&str")]
        pub enum Locale {
            $($(#[$default])? $name),*
        }

        impl From<Locale> for &str {
            fn from(locale: Locale) -> Self {
                match locale {
                    $(Locale::$name => $code),*
                }
            }
        }

        impl TryFrom<&str> for Locale {
            type Error = InvalidLocale;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $($code => Ok(Locale::$name)),*,
                    _ => Err(InvalidLocale),
                }
            }
        }

        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        $(Locale::$name => $code),*
                    }
                )
            }
        }

    }
}

make_locale! {
    Danish = "da",
    German = "de",
    EnglishUK = "en-GB",
    #[default]
    EnglishUS = "en-US",
    Spanish = "es-ES",
    French = "fr",
    Croatian = "hr",
    Italian = "it",
    Lithuanian = "lt",
    Hungarian = "hu",
    Dutch = "nl",
    Norwegian = "no",
    Polish = "pl",
    PortugueseBrazilian = "pt-BR",
    RomanianRomania = "ro",
    Finnish = "fi",
    Swedish = "sv-SE",
    Vietnamese = "vi",
    Turkish = "tr",
    Czech = "cs",
    Greek = "el",
    Bulgarian = "bg",
    Russian = "ru",
    Ukrainian = "uk",
    Hindi = "hi",
    Thai = "th",
    ChineseChina = "zh-CN",
    Japanese = "ja",
    ChineseTaiwan = "zh-TW",
    Korean = "ko",
}

#[derive(Debug, Eq, PartialEq)]
pub struct InvalidLocale;

impl std::error::Error for InvalidLocale {}

impl std::fmt::Display for InvalidLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid locale")
    }
}

#[derive(Debug)]
pub enum L10nLoadError {
    IO(std::io::Error),
    Parse(serde_yaml::Error),
    DuplicateCommand { command: String },
    DuplicateChoice { choice: String },
}

impl From<std::io::Error> for L10nLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<serde_yaml::Error> for L10nLoadError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Parse(error)
    }
}

impl std::error::Error for L10nLoadError {}

impl std::fmt::Display for L10nLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L10nLoadError::IO(error) => error.fmt(f),
            L10nLoadError::Parse(error) => error.fmt(f),
            L10nLoadError::DuplicateCommand { command } => {
                write!(f, "duplicate command {command}")
            }
            L10nLoadError::DuplicateChoice { choice } => {
                write!(f, "duplicate choice {choice}")
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct L10nLoadErrors(Vec<L10nLoadError>);

impl std::error::Error for L10nLoadErrors {}

impl std::fmt::Display for L10nLoadErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_fold((), |_, error| {
            error.fmt(f)?;
            writeln!(f)
        })
    }
}

#[derive(Debug)]
pub enum L10nStubError {
    MismatchedOptions,
}

impl std::error::Error for L10nStubError {}

impl std::fmt::Display for L10nStubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L10nStubError::MismatchedOptions => {
                writeln!(f, "mismatched options ")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPathRef<'a> {
    Command {
        name: &'a str,
    },
    Subcommand {
        name: &'a str,
        subcommand: &'a str,
    },
    Grouped {
        name: &'a str,
        group: &'a str,
        subcommand: &'a str,
    },
}

impl<'a> CommandPathRef<'a> {
    pub(crate) fn subcommand(self) -> &'a str {
        match self {
            CommandPathRef::Command { name: subcommand }
            | CommandPathRef::Subcommand { subcommand, .. }
            | CommandPathRef::Grouped { subcommand, .. } => subcommand,
        }
    }
}

impl L10n {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_yaml(content: &str) -> Result<Self, L10nLoadError> {
        serde_yaml::from_str(content).map_err(L10nLoadError::Parse)
    }

    pub fn to_yaml(&self) -> serde_yaml::Result<String> {
        serde_yaml::to_string(self)
    }

    pub async fn from_yaml_file(filename: &str) -> Result<Self, L10nLoadError> {
        tokio::fs::read_to_string(filename)
            .await
            .map_err(L10nLoadError::IO)
            .and_then(|content| Self::from_yaml(&content))
    }

    pub async fn from_yaml_files(
        filenames: impl Iterator<Item = &str>,
    ) -> Result<Self, L10nLoadErrors> {
        let (l10n, errors) = join_all(
            filenames
                .map(|filename| async move { (filename, Self::from_yaml_file(filename).await) }),
        )
        .await
        .into_iter()
        .fold(
            (Self::default(), L10nLoadErrors::default()),
            |(mut acc, mut errors), (_filename, l10n)| {
                match l10n {
                    Ok(l10n) => {
                        if let Err(merge_errors) = acc.merge(l10n) {
                            errors.0.extend(merge_errors.0)
                        }
                    }
                    Err(error) => {
                        errors.0.push(error);
                    }
                }
                (acc, errors)
            },
        );
        if errors.0.is_empty() {
            Ok(l10n)
        } else {
            Err(errors)
        }
    }

    pub fn merge(&mut self, other: Self) -> Result<(), L10nLoadErrors> {
        let mut errors = L10nLoadErrors::default();
        errors.0.extend(
            other
                .commands
                .into_iter()
                .filter_map(|(command_name, translation)| {
                    match self.commands.entry(command_name) {
                        Entry::Vacant(entry) => {
                            entry.insert(translation);
                            None
                        }
                        Entry::Occupied(entry) => Some(L10nLoadError::DuplicateCommand {
                            command: entry.key().clone(),
                        }),
                    }
                }),
        );
        errors.0.extend(
            other
                .choices
                .into_iter()
                .filter_map(
                    |(choice_name, translation)| match self.choices.entry(choice_name) {
                        Entry::Vacant(entry) => {
                            entry.insert(translation);
                            None
                        }
                        Entry::Occupied(entry) => Some(L10nLoadError::DuplicateChoice {
                            choice: entry.key().clone(),
                        }),
                    },
                ),
        );
        if errors.0.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn command_stubs(
        command_map: &CommandMap,
        locales: EnumSet<Locale>,
    ) -> Result<Self, L10nStubError> {
        let mut l10n = Self::new();
        l10n.fill_command_stubs(command_map, locales)?;
        Ok(l10n)
    }

    pub fn fill_command_stubs(
        &mut self,
        command_map: &CommandMap,
        locales: EnumSet<Locale>,
    ) -> Result<(), L10nStubError> {
        for (name, command_map_entry) in command_map.iter() {
            self.commands
                .entry(name.clone())
                .or_default()
                .fill_stubs_from_command_map_entry(command_map_entry, locales)?;
        }
        Ok(())
    }

    pub fn choice_stubs<T: Choices>(locales: EnumSet<Locale>) -> Self {
        let mut l10n = Self::new();
        l10n.choices.insert(
            T::name(),
            ChoiceL10n(
                T::choices()
                    .into_iter()
                    .map(|choice| (choice.name, Translations::stubs(locales)))
                    .collect(),
            ),
        );
        l10n
    }

    fn resolve_command_name(&self, name: &str) -> Option<&CommandL10n> {
        self.commands.get(name)
    }

    fn resolve_command_path(&self, path: CommandPathRef) -> Option<&CommandL10n> {
        match path {
            CommandPathRef::Command { name } => self.resolve_command_name(name),
            CommandPathRef::Subcommand { name, subcommand } => self
                .resolve_command_name(name)
                .and_then(|command_translations| command_translations.subcommands.get(subcommand)),
            CommandPathRef::Grouped {
                name,
                group,
                subcommand,
            } => self
                .resolve_command_name(name)
                .and_then(|command_translations| command_translations.subcommands.get(group))
                .and_then(|command_translations| command_translations.subcommands.get(subcommand)),
        }
    }

    fn resolve_command_option(
        &self,
        path: CommandPathRef,
        command_name: &str,
    ) -> Option<&OptionL10n> {
        self.resolve_command_path(path).and_then(|translations| {
            translations
                .options
                .iter()
                .find_map(|(name, option)| (name == command_name).then_some(option))
        })
    }

    pub(crate) fn describe_command(&self, name: &str, command: &mut CreateApplicationCommand) {
        let translations = self.resolve_command_name(name);
        let description =
            translation_or_default(translations.map(|translations| &translations.description));

        command.name(name).description(description);

        if let Some(translations) = translations {
            for (locale, translation) in &translations.name.0 {
                command.name_localized(locale, translation);
            }
            for (locale, translation) in &translations.description.0 {
                command.description_localized(locale, translation);
            }
        }
    }

    pub(crate) fn describe_subcommand(
        &self,
        path: CommandPathRef,
        option: &mut CreateApplicationCommandOption,
    ) {
        let translations = self.resolve_command_path(path);
        let description =
            translation_or_default(translations.map(|translations| &translations.description));

        option.name(path.subcommand()).description(description);

        if let Some(translations) = translations {
            for (locale, translation) in &translations.name.0 {
                option.name_localized(locale, translation);
            }
            for (locale, translation) in &translations.description.0 {
                option.description_localized(locale, translation);
            }
        }
    }

    pub fn describe_command_option(
        &self,
        path: CommandPathRef,
        name: &str,
        option: &mut CreateApplicationCommandOption,
    ) {
        let translations = self.resolve_command_option(path, name);
        let description =
            translation_or_default(translations.map(|translations| &translations.description));

        option.name(name).description(description);

        if let Some(translations) = translations {
            for (locale, translation) in &translations.name.0 {
                option.name_localized(locale, translation);
            }
            for (locale, translation) in &translations.description.0 {
                option.description_localized(locale, translation);
            }
        }
    }

    pub(crate) fn describe_string_choice(
        &self,
        name: &str,
        choice: impl Into<String>,
        value: impl Into<String>,
        option: &mut CreateApplicationCommandOption,
    ) {
        let choice = choice.into();
        let translations = self.choices.get(name).and_then(|translations| {
            translations
                .0
                .iter()
                .find_map(|(choice_name, translations)| {
                    (choice_name == &choice).then_some(translations)
                })
        });
        match translations {
            Some(translations) => {
                // TODO: Remove &choice and value.into() in next major serenity release.
                option.add_string_choice_localized(&choice, value.into(), &translations.0);
            }
            None => {
                // TODO: Remove value.into() in next major serenity release.
                option.add_string_choice(choice, value.into());
            }
        }
    }
}

impl CommandL10n {
    fn fill_stubs_from_command_map_entry(
        &mut self,
        command: &CommandMapEntry,
        locales: EnumSet<Locale>,
    ) -> Result<(), L10nStubError> {
        self.name.fill_stubs(locales);
        self.description.fill_stubs(locales);
        match command {
            CommandMapEntry::Command(command) => {
                self.fill_stubs_from_command(command, locales)?;
            }
            CommandMapEntry::Subcommands(subcommands) => {
                for (name, subcommand) in subcommands {
                    self.subcommands
                        .entry(name.clone())
                        .or_default()
                        .fill_stubs_from_subcommand_map_entry(subcommand, locales)?;
                }
            }
        }
        Ok(())
    }

    fn fill_stubs_from_subcommand_map_entry(
        &mut self,
        subcommand: &SubcommandMapEntry,
        locales: EnumSet<Locale>,
    ) -> Result<(), L10nStubError> {
        self.name.fill_stubs(locales);
        self.description.fill_stubs(locales);
        match subcommand {
            SubcommandMapEntry::Subcommand(command) => {
                self.fill_stubs_from_command(command, locales)?;
            }
            SubcommandMapEntry::Group(group) => {
                for (name, command) in group {
                    self.subcommands
                        .entry(name.clone())
                        .or_default()
                        .fill_stubs_from_command(command, locales)?;
                }
            }
        }
        Ok(())
    }

    fn fill_stubs_from_command(
        &mut self,
        command: impl AsRef<dyn Command>,
        locales: EnumSet<Locale>,
    ) -> Result<(), L10nStubError> {
        let command = command.as_ref();

        self.name.fill_stubs(locales);
        self.description.fill_stubs(locales);

        let new_options = command.options();
        let mut new_options = new_options.iter();

        for ((current_name, current_option), name) in zip(self.options.iter_mut(), &mut new_options)
        {
            if name != current_name {
                Err(L10nStubError::MismatchedOptions)?;
            } else {
                current_option.fill_stubs(locales);
            }
        }

        self.options
            .extend(new_options.map(|option| (option.clone(), OptionL10n::stubs(locales))));

        Ok(())
    }
}

impl Translations {
    fn new() -> Self {
        Self::default()
    }

    fn stubs(locales: EnumSet<Locale>) -> Self {
        let mut translations = Self::new();
        translations.fill_stubs(locales);
        translations
    }

    fn fill_stubs(&mut self, locales: EnumSet<Locale>) {
        for locale in locales {
            self.0.entry(locale).or_insert_with(|| "TODO".to_string());
        }
    }
}

fn translation_or_default(translations: Option<&Translations>) -> &str {
    translations
        .and_then(|translations| translations.0.get(&Locale::default()).map(AsRef::as_ref))
        .unwrap_or("n/a")
}
