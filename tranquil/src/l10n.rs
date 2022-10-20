use std::collections::{hash_map::Entry, HashMap};

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};

#[derive(Debug, Default, Eq, PartialEq)]
struct InvalidLocale;

impl std::error::Error for InvalidLocale {}

impl std::fmt::Display for InvalidLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid locale")
    }
}

macro_rules! make_locale {
    {$($(#[$default:ident])? $name:ident = $code:literal),* $(,)?} => {
        #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(into = "&str", try_from = "&str")]
        enum Locale {
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

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
struct Translations(HashMap<Locale, String>);

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OptionTranslations {
    #[serde(default)]
    name: Translations,
    #[serde(default)]
    description: Translations,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct CommandTranslations {
    #[serde(default)]
    name: Translations,
    #[serde(default)]
    description: Translations,
    #[serde(default)]
    subcommands: HashMap<String, CommandTranslations>,
    #[serde(default)]
    options: HashMap<String, OptionTranslations>,
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TranslatedCommands(HashMap<String, CommandTranslations>);

#[derive(Debug)]
pub(crate) enum L10nLoadError {
    IO(std::io::Error),
    Parse(toml::de::Error),
    DuplicateCommand { filename: String },
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
            L10nLoadError::DuplicateCommand { filename } => {
                write!(f, "duplicate command in {filename}")
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

impl TranslatedCommands {
    pub(crate) async fn from_file(filename: &str) -> Result<Self, L10nLoadError> {
        Ok(toml::from_str(&tokio::fs::read_to_string(filename).await?)?)
    }

    pub(crate) async fn from_files(
        filenames: impl Iterator<Item = &str>,
    ) -> Result<Self, L10nLoadErrors> {
        let (translations, errors) = join_all(
            filenames.map(|filename| async move { (filename, Self::from_file(filename).await) }),
        )
        .await
        .into_iter()
        .fold(
            (Self::default(), L10nLoadErrors::default()),
            |(mut acc, mut errors), (filename, translations)| {
                match translations {
                    Ok(translations) => {
                        errors.0.extend(translations.0.into_iter().filter_map(
                            |(command_name, translation)| {
                                if let Entry::Vacant(entry) = acc.0.entry(command_name) {
                                    entry.insert(translation);
                                    None
                                } else {
                                    Some(L10nLoadError::DuplicateCommand {
                                        filename: filename.to_string(),
                                    })
                                }
                            },
                        ));
                    }
                    Err(error) => {
                        errors.0.push(error);
                    }
                }
                (acc, errors)
            },
        );
        if errors.0.is_empty() {
            Ok(translations)
        } else {
            Err(errors)
        }
    }

    fn resolve_name(&self, name: &str) -> Option<&CommandTranslations> {
        self.0.get(name)
    }

    fn resolve_path(&self, path: CommandPathRef) -> Option<&CommandTranslations> {
        match path {
            CommandPathRef::Command { name } => self.resolve_name(name),
            CommandPathRef::Subcommand { name, subcommand } => self
                .resolve_name(name)
                .and_then(|command_translations| command_translations.subcommands.get(subcommand)),
            CommandPathRef::Grouped {
                name,
                group,
                subcommand,
            } => self
                .resolve_name(name)
                .and_then(|command_translations| command_translations.subcommands.get(group))
                .and_then(|command_translations| command_translations.subcommands.get(subcommand)),
        }
    }

    fn resolve_option(&self, path: CommandPathRef, name: &str) -> Option<&OptionTranslations> {
        self.resolve_path(path)
            .and_then(|translations| translations.options.get(name))
    }

    pub(crate) fn describe_command(&self, name: &str, command: &mut CreateApplicationCommand) {
        let translations = self.resolve_name(name);
        let description =
            get_default_translation(translations.map(|translations| &translations.description));

        command.name(name).description(description);

        if let Some(translations) = translations {
            for (locale, translation) in &translations.name.0 {
                command.name_localized(locale.to_string(), translation);
            }
            for (locale, translation) in &translations.description.0 {
                command.description_localized(locale.to_string(), translation);
            }
        }
    }

    pub(crate) fn describe_subcommand(
        &self,
        path: CommandPathRef,
        option: &mut CreateApplicationCommandOption,
    ) {
        let translations = self.resolve_path(path);
        let description =
            get_default_translation(translations.map(|translations| &translations.description));

        option.name(path.subcommand()).description(description);

        if let Some(translations) = translations {
            for (locale, translation) in &translations.name.0 {
                option.name_localized(locale.to_string(), translation);
            }
            for (locale, translation) in &translations.description.0 {
                option.description_localized(locale.to_string(), translation);
            }
        }
    }

    pub fn describe_option(
        &self,
        path: CommandPathRef,
        name: &str,
        option: &mut CreateApplicationCommandOption,
    ) {
        let translations = self.resolve_option(path, name);
        let description =
            get_default_translation(translations.map(|translations| &translations.description));

        option.name(name).description(description);

        if let Some(translations) = translations {
            for (locale, translation) in &translations.name.0 {
                option.name_localized(locale.to_string(), translation);
            }
            for (locale, translation) in &translations.description.0 {
                option.description_localized(locale.to_string(), translation);
            }
        }
    }
}

pub trait CommandL10nProvider {
    fn l10n_path(&self) -> Option<&'static str> {
        None
    }
}
