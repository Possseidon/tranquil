pub mod autocomplete;
pub mod bot;
pub mod command;
pub mod l10n;
pub mod module;
pub mod resolve;
pub mod utils;

pub mod macros {
    pub use tranquil_macros::{autocompleter, command_provider, slash};
}

extern crate self as tranquil;

pub use anyhow;
pub use async_trait::async_trait;
pub use serenity;
