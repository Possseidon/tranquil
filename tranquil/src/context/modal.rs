use serenity::{
    self,
    builder::{
        CreateInteractionResponse, CreateInteractionResponseFollowup, EditInteractionResponse,
    },
    client::Context,
    model::{
        application::interaction::modal::ModalSubmitInteraction, channel::Message, id::MessageId,
    },
};

pub struct ModalCtx {
    pub bot: Context,
    pub interaction: ModalSubmitInteraction,
}

pub struct ModalCtxWithResponse {
    pub bot: Context,
    pub interaction: ModalSubmitInteraction,
}

pub struct ModalCtxWithDeletedResponse {
    pub bot: Context,
    pub interaction: ModalSubmitInteraction,
}

#[derive(Clone, Copy)]
pub struct ModalCtxFollowups<'a> {
    pub bot: &'a Context,
    pub interaction: &'a ModalSubmitInteraction,
}

impl ModalCtx {
    pub async fn respond<'a, F>(self, f: F) -> serenity::Result<ModalCtxWithResponse>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        self.interaction
            .create_interaction_response(&self.bot, f)
            .await?;
        Ok(ModalCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub async fn defer(self) -> serenity::Result<ModalCtxWithResponse> {
        self.interaction.defer(&self.bot).await?;
        Ok(ModalCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub async fn defer_ephemeral(self) -> serenity::Result<ModalCtxWithResponse> {
        self.interaction.defer_ephemeral(&self.bot).await?;
        Ok(ModalCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }
}

impl ModalCtxWithResponse {
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

    pub async fn delete_response(self) -> serenity::Result<ModalCtxWithDeletedResponse> {
        self.interaction
            .delete_original_interaction_response(&self.bot)
            .await?;
        Ok(ModalCtxWithDeletedResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub fn followups(&self) -> ModalCtxFollowups {
        ModalCtxFollowups {
            bot: &self.bot,
            interaction: &self.interaction,
        }
    }
}

impl ModalCtxWithDeletedResponse {
    pub fn followups(&self) -> ModalCtxFollowups {
        ModalCtxFollowups {
            bot: &self.bot,
            interaction: &self.interaction,
        }
    }
}

impl ModalCtxFollowups<'_> {
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
        // TODO: Does this work?
        self.bot
            .http
            .get_followup_message(&self.interaction.token, message_id.into())
            .await
    }
}
