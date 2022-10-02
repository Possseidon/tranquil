pub mod bot;
pub mod module;
pub mod resolve;
pub mod slash_command;

pub use tranquil_macros::*;
extern crate self as tranquil;

pub use serenity;

pub type AnyError = Box<dyn std::error::Error + Send + Sync>;
pub type AnyResult<T> = Result<T, AnyError>;
