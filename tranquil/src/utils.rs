use std::{env, ffi::OsStr};

use anyhow::Result;
use serenity::model::id::GuildId;

use crate::bot::ApplicationCommandUpdate;

pub fn dotenv_if_exists() -> Result<(), dotenvy::Error> {
    match dotenvy::dotenv() {
        Err(error) if error.not_found() => Ok(()),
        Err(error) => Err(error),
        Ok(_) => Ok(()),
    }
}

pub fn discord_token_from_env() -> Result<String, env::VarError> {
    discord_token_from_env_var("DISCORD_TOKEN")
}

pub fn debug_guilds_from_env() -> Result<Option<ApplicationCommandUpdate>> {
    debug_guilds_from_env_var("DEBUG_GUILDS")
}

pub fn discord_token_from_env_var(key: impl AsRef<OsStr>) -> Result<String, env::VarError> {
    env::var(&key).map_err(|error| {
        if let env::VarError::NotPresent = error {
            eprintln!(
                "{} environment variable not found",
                key.as_ref().to_string_lossy()
            );
        }
        error
    })
}

pub fn debug_guilds_from_env_var(
    key: impl AsRef<OsStr>,
) -> Result<Option<ApplicationCommandUpdate>> {
    debug_guilds_from_env_var_silent(&key).map_err(|error| {
        eprintln!("{} invalid", key.as_ref().to_string_lossy());
        error
    })
}

fn debug_guilds_from_env_var_silent(
    key: impl AsRef<OsStr>,
) -> Result<Option<ApplicationCommandUpdate>> {
    match env::var(key) {
        Ok(debug_guilds) => Ok(Some(ApplicationCommandUpdate::Only({
            let parsed_guild_ids = debug_guilds
                .trim()
                .split_terminator(',')
                .map(|guild_id| guild_id.trim().parse::<u64>().map(GuildId));
            let mut guild_ids = vec![];
            for guild_id in parsed_guild_ids {
                guild_ids.push(guild_id?);
            }
            guild_ids
        }))),
        Err(error) => match error {
            env::VarError::NotPresent => Ok(Some(ApplicationCommandUpdate::default())),
            _ => Err(error)?,
        },
    }
}
