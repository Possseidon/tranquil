use serenity::all::{ChannelId, CommandDataOptionValue, CommandOptionType};

use super::{CommandOption, ResolveRequired, Unvalidated, autocomplete::NotAutocompletable};
use crate::interaction::error::Result;

impl CommandOption for ChannelId {
    const KIND: CommandOptionType = CommandOptionType::Channel;
}

impl Unvalidated for ChannelId {
    type Unvalidated = NotAutocompletable;
}

impl ResolveRequired for ChannelId {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_channel_id()
            .ok_or(Self::unexpected_type(data).into())
    }
}

// TODO: macro to create a channel limited to specific types
