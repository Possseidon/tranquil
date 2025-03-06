use std::ops::Deref;

use serenity::all::{CommandDataOptionValue, CommandOptionType, GenericId};

use super::{CommandOption, ResolveRequired, Unvalidated, autocomplete::NotAutocompletable};
use crate::interaction::error::Result;

/// A user or role.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MentionableId(GenericId);

impl Deref for MentionableId {
    type Target = GenericId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CommandOption for MentionableId {
    const KIND: CommandOptionType = CommandOptionType::Mentionable;
}

impl Unvalidated for MentionableId {
    type Unvalidated = NotAutocompletable;
}

impl ResolveRequired for MentionableId {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_mentionable()
            .map(MentionableId)
            .ok_or(Self::unexpected_type(data).into())
    }
}
