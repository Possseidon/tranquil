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

impl TranslatedCommands {
    pub(crate) async fn from_file(filename: &str) -> Result<Self, L10nLoadError> {
        toml::from_str(
            &tokio::fs::read_to_string(filename)
                .await
                .map_err(L10nLoadError::IO)?,
        )
        .map_err(L10nLoadError::Parse)
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

    pub(crate) fn describe_command(
        &self,
        command_name: &str,
        command: &mut CreateApplicationCommand,
    ) {
        command
            .name(&command_name)
            .description(get_default_translation({
                self.0
                    .get(command_name)
                    .map(|translations| &translations.description)
            }));
        if let Some(translations) = self.0.get(command_name) {
            for (locale, translation) in &translations.name.0 {
                command.name_localized(locale.to_string(), translation);
            }
            for (locale, translation) in &translations.description.0 {
                command.description_localized(locale.to_string(), translation);
            }
        }
    }

    pub fn describe_option(
        &self,
        command_name: &str,
        command_option_name: &str,
        option: &mut CreateApplicationCommandOption,
    ) {
        option
            .name(command_option_name)
            .description(get_default_translation({
                self.0
                    .get(command_name)
                    .map(|command_translations| &command_translations.options)
                    .and_then(|option_translations| option_translations.get(command_option_name))
                    .map(|option_translation| &option_translation.description)
            }));
        if let Some(command_translations) = self.0.get(command_name) {
            if let Some(option_translations) = command_translations.options.get(command_option_name)
            {
                for (locale, translation) in &option_translations.name.0 {
                    option.name_localized(locale, translation);
                }
                for (locale, translation) in &option_translations.description.0 {
                    option.description_localized(locale, translation);
                }
            }
        }
    }
}

pub trait CommandL10nProvider {
    fn translations_filepath(&self) -> &'static str;
}
