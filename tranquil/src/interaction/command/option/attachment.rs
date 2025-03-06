use serenity::all::{AttachmentId, CommandDataOptionValue, CommandOptionType};

use super::{CommandOption, ResolveRequired, Unvalidated, autocomplete::NotAutocompletable};
use crate::interaction::error::Result;

impl CommandOption for AttachmentId {
    const KIND: CommandOptionType = CommandOptionType::Attachment;
}

impl Unvalidated for AttachmentId {
    type Unvalidated = NotAutocompletable;
}

impl ResolveRequired for AttachmentId {
    fn resolve(data: &mut CommandDataOptionValue) -> Result<Self> {
        data.as_attachment_id()
            .ok_or(Self::unexpected_type(data).into())
    }
}
