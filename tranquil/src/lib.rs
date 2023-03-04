pub mod autocomplete;
pub mod bot;
pub mod button;
pub mod command;
pub mod context;
pub mod custom_id;
pub mod interaction;
pub mod l10n;
pub mod modal;
pub mod module;
pub mod resolve;
pub mod select_menu;
pub mod utils;

pub mod macros {
    pub use tranquil_macros::{autocompleter, command_provider, slash};
}

extern crate self as tranquil;

// Re-exports for macros

#[doc(hidden)]
pub use anyhow;
#[doc(hidden)]
pub use async_trait::async_trait;
#[doc(hidden)]
pub use enumset;
#[doc(hidden)]
pub use serenity;
#[doc(hidden)]
pub use uuid;
