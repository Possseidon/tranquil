use delegate::delegate;
use serenity::{
    builder::{
        CreateInteractionResponse, CreateInteractionResponseFollowup, EditInteractionResponse,
    },
    client::Context,
    model::{
        application::interaction::modal::ModalSubmitInteraction, channel::Message, id::MessageId,
    },
};

pub struct ModalContext {
    pub bot: Context,
    pub interaction: ModalSubmitInteraction,
}

impl ModalContext {
    delegate! {
        to self.interaction {
            pub async fn get_interaction_response(&self, [ &self.bot ]) -> serenity::Result<Message>;

            pub async fn create_interaction_response<'a, F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<()>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponse<'a>,
                ) -> &'b mut CreateInteractionResponse<'a>;

            pub async fn edit_original_interaction_response<F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<Message>
            where
                F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse;

            pub async fn delete_original_interaction_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<()>;

            pub async fn create_followup_message<'a, F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            pub async fn edit_followup_message<'a, F>(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
                f: F,
            ) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            pub async fn delete_followup_message(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<()>;

            pub async fn defer(&self, [ &self.bot ]) -> serenity::Result<()>;
        }
    }
}
