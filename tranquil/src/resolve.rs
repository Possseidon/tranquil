use std::sync::Arc;

use serenity::{
    async_trait,
    builder::CreateApplicationCommandOption,
    http::Http,
    model::{
        application::{
            command::CommandOptionType,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        },
        channel::{Attachment, PartialChannel},
        guild::Role,
    },
};

use crate::l10n::L10n;

#[derive(Clone, Debug)]
pub struct ResolveContext {
    pub option: Option<CommandDataOption>,
    pub http: Arc<Http>,
}

#[async_trait]
pub trait Resolve: Sized {
    const KIND: CommandOptionType;
    const REQUIRED: bool = true;

    fn describe(_option: &mut CreateApplicationCommandOption, _l10n: &L10n) {}

    async fn resolve(ctx: ResolveContext) -> error::ResolveResult<Self>;
}

macro_rules! impl_resolve {
    ($($command_option_type:ident => $t:ty),* $(,)?) => { $(
        #[async_trait]
        impl Resolve for $t {
            const KIND: CommandOptionType = CommandOptionType::$command_option_type;

            async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
                match resolve_option(ctx.option)? {
                    CommandDataOptionValue::$command_option_type(value) => Ok(value),
                    _ => Err(ResolveError::InvalidType),
                }
            }
        }
    )* };
}

impl_resolve! {
    String => String,
    Integer => i64,
    Boolean => bool,
    Channel => PartialChannel,
    Role => Role,
    Number => f64,
    Attachment => Attachment,
}

mod channel;
mod error;
mod integer;
mod mentionable;
mod number;
mod option;
mod string;
mod user;

pub use channel::*;
pub use error::*;
pub use mentionable::*;
pub use string::*;

fn resolve_option(
    option: Option<CommandDataOption>,
) -> error::ResolveResult<CommandDataOptionValue> {
    option.map_or(Err(error::ResolveError::Missing), |option| {
        option.resolved.ok_or(error::ResolveError::Unresolvable)
    })
}

pub fn find_options<'a>(
    names: impl IntoIterator<Item = &'a str>,
    options: impl IntoIterator<Item = CommandDataOption>,
) -> Vec<Option<CommandDataOption>> {
    let mut options: Vec<Option<CommandDataOption>> = options.into_iter().map(Some).collect();
    names
        .into_iter()
        .map(|name| {
            options
                .iter_mut()
                .find_map(|option| match option {
                    Some(CommandDataOption {
                        name: option_name, ..
                    }) if option_name == name => option.take(),
                    _ => None,
                })
                .take()
        })
        .collect()
}

pub fn resolve_command_options(mut options: Vec<CommandDataOption>) -> Vec<CommandDataOption> {
    if options.len() != 1 {
        options
    } else if options[0].kind == CommandOptionType::SubCommand
        || options[0].kind == CommandOptionType::SubCommandGroup
    {
        let mut group = options.remove(0);
        if group.options.len() != 1 {
            group.options
        } else if group.options[0].kind == CommandOptionType::SubCommand {
            group.options.remove(0).options
        } else {
            group.options
        }
    } else {
        options
    }
}
