use serenity::{
    async_trait,
    model::{
        application::{
            command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
        },
        guild::PartialMember,
        user::User,
    },
};

use super::{resolve_option, Resolve, ResolveContext, ResolveError, ResolveResult};

#[async_trait]
impl Resolve for User {
    const KIND: CommandOptionType = CommandOptionType::User;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::User(value, _) => Ok(value),
            _ => Err(ResolveError::InvalidType),
        }
    }
}

#[async_trait]
impl Resolve for PartialMember {
    const KIND: CommandOptionType = CommandOptionType::User;

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match resolve_option(ctx.option)? {
            CommandDataOptionValue::User(_, Some(value)) => Ok(value),
            CommandDataOptionValue::User(_, None) => Err(ResolveError::NoPartialMemberData),
            _ => Err(ResolveError::InvalidType),
        }
    }
}
