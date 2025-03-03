use async_trait::async_trait;
use serenity::model::{
    application::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
    guild::{PartialMember, Role},
    mention::Mention,
    user::User,
};

use super::{resolve_option, Resolve, ResolveContext, ResolveError, ResolveResult};

#[derive(Clone, Debug)]
pub enum Mentionable {
    User(User, Option<PartialMember>),
    Role(Role),
}

#[async_trait]
impl Resolve for Mentionable {
    const KIND: CommandOptionType = CommandOptionType::Mentionable;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::User(user, partial_member) => {
                Ok(Self::User(user, partial_member))
            }
            CommandDataOptionValue::Role(role) => Ok(Self::Role(role)),
            _ => Err(ResolveError::InvalidType),
        }
    }
}

#[async_trait]
impl Resolve for Mention {
    const KIND: CommandOptionType = CommandOptionType::Mentionable;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::User(user, _) => Ok(Self::User(user.id)),
            CommandDataOptionValue::Role(role) => Ok(Self::Role(role.id)),
            // Mention can also store Channels and Emojis, which are not valid for
            // CommandOptionType::Mentionable
            _ => Err(ResolveError::InvalidType),
        }
    }
}
