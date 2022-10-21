use serenity::{
    async_trait,
    model::{
        application::{
            command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
        },
        guild::{PartialMember, Role},
        user::User,
    },
};

use super::{resolve_option, Resolve, ResolveContext, ResolveError, ResolveResult};

#[derive(Debug)]
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
                Ok(Mentionable::User(user, partial_member))
            }
            CommandDataOptionValue::Role(role) => Ok(Mentionable::Role(role)),
            _ => Err(ResolveError::InvalidType),
        }
    }
}
