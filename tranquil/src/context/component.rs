use serenity::{
    self,
    builder::{
        CreateInteractionResponse, CreateInteractionResponseFollowup, EditInteractionResponse,
    },
    client::Context,
    model::{
        application::interaction::message_component::MessageComponentInteraction, channel::Message,
        id::MessageId,
    },
};

pub struct ComponentCtx {
    pub bot: Context,
    pub interaction: MessageComponentInteraction,
}

pub struct ComponentCtxWithResponse {
    pub bot: Context,
    pub interaction: MessageComponentInteraction,
}

pub struct ComponentCtxWithDeletedResponse {
    pub bot: Context,
    pub interaction: MessageComponentInteraction,
}

#[derive(Clone, Copy)]
pub struct ComponentCtxFollowups<'a> {
    pub bot: &'a Context,
    pub interaction: &'a MessageComponentInteraction,
}

impl ComponentCtx {
    pub async fn respond<'a, F>(self, f: F) -> serenity::Result<ComponentCtxWithResponse>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        self.interaction
            .create_interaction_response(&self.bot, f)
            .await?;
        Ok(ComponentCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub async fn defer(self) -> serenity::Result<ComponentCtxWithResponse> {
        self.interaction.defer(&self.bot).await?;
        Ok(ComponentCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub async fn defer_ephemeral(self) -> serenity::Result<ComponentCtxWithResponse> {
        self.interaction.defer_ephemeral(&self.bot).await?;
        Ok(ComponentCtxWithResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }
}

impl ComponentCtxWithResponse {
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

    pub async fn delete_response(self) -> serenity::Result<ComponentCtxWithDeletedResponse> {
        self.interaction
            .delete_original_interaction_response(&self.bot)
            .await?;
        Ok(ComponentCtxWithDeletedResponse {
            bot: self.bot,
            interaction: self.interaction,
        })
    }

    pub fn followups(&self) -> ComponentCtxFollowups {
        ComponentCtxFollowups {
            bot: &self.bot,
            interaction: &self.interaction,
        }
    }
}

impl ComponentCtxWithDeletedResponse {
    pub fn followups(&self) -> ComponentCtxFollowups {
        ComponentCtxFollowups {
            bot: &self.bot,
            interaction: &self.interaction,
        }
    }
}

impl ComponentCtxFollowups<'_> {
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
