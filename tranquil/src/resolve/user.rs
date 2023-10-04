use async_trait::async_trait;
use serenity::model::{
    application::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
    guild::PartialMember,
    user::User,
};

use super::{resolve_option, Resolve, ResolveContext, ResolveError, ResolveResult};

#[async_trait]
impl Resolve for User {
    const KIND: CommandOptionType = CommandOptionType::User;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::User(value, _) => Ok(value),
            _ => Err(ResolveError::InvalidType.into()),
        }
    }
}

#[async_trait]
impl Resolve for PartialMember {
    const KIND: CommandOptionType = CommandOptionType::User;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::User(_, Some(value)) => Ok(value),
            CommandDataOptionValue::User(_, None) => Err(ResolveError::NoPartialMemberData.into()),
            _ => Err(ResolveError::InvalidType.into()),
        }
    }
}
