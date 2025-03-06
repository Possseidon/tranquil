use serenity::all::{CommandOptionType, CreateInteractionResponseMessage, GenericId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Discord(#[from] DiscordError),
    /// An error caused by internal logic errors, indicating a bug.
    ///
    /// In essence, these are similar to a [`panic`] or failed [`assert`]ion. The main difference is
    /// that they are still handled gracefully and send the user an "Internal Error" response.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// The user entered an invalid value.
///
/// A response containing detailed information about what the user did wrong is automatically
/// sent as an ephemeral message.
///
/// Only causes an `INFO` level log, since these are to be expected.
#[derive(Debug, Error)]
#[error("user error")]
pub struct UserError(pub Box<CreateInteractionResponseMessage>);

/// An error due to an unexpected value from the Discord API.
///
/// E.g. a value that does not exist despite being required or.
#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("command option should be required but is missing")]
    MissingRequiredOption,
    #[error("command option should be {expected:?}, got {got:?}")]
    UnexpectedOptionType {
        expected: CommandOptionType,
        got: CommandOptionType,
    },
    #[error("command option should be between {min} and {max} characters, got {got}")]
    StringLengthOutOfBounds { min: u16, max: u16, got: usize },
    #[error("command option should not be autocompletable")]
    NotAutocompletable,
}

pub type Result<T> = std::result::Result<T, Error>;
