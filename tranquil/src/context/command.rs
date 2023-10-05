use serenity::{
    self,
    builder::{
        CreateInteractionResponse, CreateInteractionResponseFollowup, EditInteractionResponse,
    },
    client::Context,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        channel::Message, id::MessageId,
    },
};

pub struct CommandCtx {
    pub bot: Context,
    pub interaction: ApplicationCommandInteraction,
}

pub struct CommandCtxWithResponse {
    pub bot: Context,
    pub interaction: ApplicationCommandInteraction,
}

pub struct CommandCtxWithDeletedResponse {
    pub bot: Context,
    pub interaction: ApplicationCommandInteraction,
}

#[derive(Clone, Copy)]
pub struct CommandCtxFollowups<'a> {
    pub bot: &'a Context,
    pub interaction: &'a ApplicationCommandInteraction,
}

impl CommandCtx {
    pub async fn respond<'a, F>(self, f: F) -> serenity::Result<CommandCtxWithResponse>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        self.interaction
            .create_interaction_response(&self.bot, f)
            .await?;
        Ok(CommandCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub async fn defer(self) -> serenity::Result<CommandCtxWithResponse> {
        self.interaction.defer(&self.bot).await?;
        Ok(CommandCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub async fn defer_ephemeral(self) -> serenity::Result<CommandCtxWithResponse> {
        self.interaction.defer_ephemeral(&self.bot).await?;
        Ok(CommandCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }
}

impl CommandCtxWithResponse {
    pub async fn get_response(&self) -> serenity::Result<Message> {
        self.interaction.get_interaction_response(&self.bot).await
    }

    pub async fn edit_response(
        self,
        f: impl FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    ) -> serenity::Result<Self> {
        self.interaction
            .edit_original_interaction_response(&self.bot, f)
            .await?;
        Ok(self)
    }

    pub async fn delete_response(self) -> serenity::Result<CommandCtxWithDeletedResponse> {
        self.interaction
            .delete_original_interaction_response(&self.bot)
            .await?;
        Ok(CommandCtxWithDeletedResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub fn followups(&self) -> CommandCtxFollowups {
        CommandCtxFollowups {
            bot: &self.bot,
            interaction: &self.interaction,
        }
    }
}

impl CommandCtxWithDeletedResponse {
    pub fn followups(&self) -> CommandCtxFollowups {
        CommandCtxFollowups {
            bot: &self.bot,
            interaction: &self.interaction,
        }
    }
}

impl CommandCtxFollowups<'_> {
    pub async fn create<'a, F>(self, f: F) -> serenity::Result<Message>
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseFollowup<'a>,
        ) -> &'b mut CreateInteractionResponseFollowup<'a>,
    {
        self.interaction.create_followup_message(&self.bot, f).await
    }

    pub async fn edit<'a, F>(self, message_id: MessageId, f: F) -> serenity::Result<Message>
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseFollowup<'a>,
        ) -> &'b mut CreateInteractionResponseFollowup<'a>,
    {
        self.interaction
            .edit_followup_message(&self.bot, message_id, f)
            .await
    }

    pub async fn delete(self, message_id: MessageId) -> serenity::Result<()> {
        self.interaction
            .delete_followup_message(&self.bot, message_id)
            .await
    }

    pub async fn get(self, message_id: MessageId) -> serenity::Result<Message> {
        self.interaction
            .get_followup_message(&self.bot, message_id)
            .await
    }
}
