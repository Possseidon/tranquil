use std::collections::{btree_map::Entry, BTreeMap};

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct InvalidLocale;

impl std::error::Error for InvalidLocale {}

impl std::fmt::Display for InvalidLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid locale")
    }
}

macro_rules! make_locale {
    {$($(#[$default:ident])? $name:ident = $code:literal),* $(,)?} => {
        #[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
struct Translations(BTreeMap<Locale, String>);

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OptionL10n {
    #[serde(default)]
    name: Translations,
    #[serde(default)]
    description: Translations,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CommandL10n {
    #[serde(default)]
    name: Translations,
    #[serde(default)]
    description: Translations,
    #[serde(default)]
    subcommands: BTreeMap<String, CommandL10n>,
    #[serde(default, with = "tuple_vec_map")]
    options: Vec<(String, OptionL10n)>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
struct ChoiceL10n(#[serde(with = "tuple_vec_map")] Vec<(String, Translations)>);

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct L10n {
    #[serde(default)]
    commands: BTreeMap<String, CommandL10n>,
    #[serde(default)]
    choices: BTreeMap<String, ChoiceL10n>,
}

#[derive(Debug)]
pub(crate) enum L10nLoadError {
    IO(std::io::Error),
    Parse(toml::de::Error),
    DuplicateCommand { command: String },
    DuplicateChoice { choice: String },
}

impl From<std::io::Error> for L10nLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<toml::de::Error> for L10nLoadError {
    fn from(error: toml::de::Error) -> Self {
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
pub(crate) struct L10nLoadErrors(Vec<L10nLoadError>);

impl std::error::Error for L10nLoadErrors {}

impl std::fmt::Display for L10nLoadErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_fold((), |_, error| {
            error.fmt(f)?;
            writeln!(f)
        })
    }
}

fn get_default_translation(translations: Option<&Translations>) -> &str {
    translations
        .and_then(|translations| translations.0.get(&Locale::default()).map(AsRef::as_ref))
        .unwrap_or("n/a")
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
    pub(crate) async fn from_file(filename: &str) -> Result<Self, L10nLoadError> {
        Ok(toml::from_str(&tokio::fs::read_to_string(filename).await?)?)
    }

    pub(crate) async fn from_files(
        filenames: impl Iterator<Item = &str>,
    ) -> Result<Self, L10nLoadErrors> {
        let (l10n, errors) = join_all(
            filenames.map(|filename| async move { (filename, Self::from_file(filename).await) }),
        )
        .await
        .into_iter()
        .fold(
            (Self::default(), L10nLoadErrors::default()),
            |(mut acc, mut errors), (_filename, l10n)| {
                match l10n {
                    Ok(l10n) => {
                        errors.0.extend(l10n.commands.into_iter().filter_map(
                            |(command_name, translation)| match acc.commands.entry(command_name) {
                                Entry::Vacant(entry) => {
                                    entry.insert(translation);
                                    None
                                }
                                Entry::Occupied(entry) => Some(L10nLoadError::DuplicateCommand {
                                    command: entry.key().clone(),
                                }),
                            },
                        ));
                        errors.0.extend(l10n.choices.into_iter().filter_map(
                            |(choice_name, translation)| match acc.choices.entry(choice_name) {
                                Entry::Vacant(entry) => {
                                    entry.insert(translation);
                                    None
                                }
                                Entry::Occupied(entry) => Some(L10nLoadError::DuplicateChoice {
                                    choice: entry.key().clone(),
                                }),
                            },
                        ))
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
            get_default_translation(translations.map(|translations| &translations.description));

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
            get_default_translation(translations.map(|translations| &translations.description));

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
            get_default_translation(translations.map(|translations| &translations.description));

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

pub trait CommandL10nProvider {
    fn l10n_path(&self) -> Option<&str> {
        None
    }
}
