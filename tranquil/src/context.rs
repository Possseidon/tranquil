use delegate::delegate;
use serenity::{
    builder::{
        CreateAutocompleteResponse, CreateInteractionResponse, CreateInteractionResponseFollowup,
        EditInteractionResponse,
    },
    client::Context,
    http::{CacheHttp, Http},
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction,
            autocomplete::AutocompleteInteraction, message_component::MessageComponentInteraction,
            modal::ModalSubmitInteraction,
        },
        channel::Message,
        id::MessageId,
    },
};

pub struct Ctx<T> {
    pub bot: Context,
    pub interaction: T,
}

impl<T> AsRef<Http> for Ctx<T> {
    fn as_ref(&self) -> &Http {
        self.bot.as_ref()
    }
}

impl<T: Send + Sync> CacheHttp for Ctx<T> {
    fn http(&self) -> &Http {
        self.bot.http()
    }
}

pub type CommandCtx = Ctx<ApplicationCommandInteraction>;
pub type AutocompleteCtx = Ctx<AutocompleteInteraction>;
pub type MessageComponentCtx = Ctx<MessageComponentInteraction>;
pub type ModalCtx = Ctx<ModalSubmitInteraction>;

impl CommandCtx {
    delegate! {
        to self.interaction {
            pub async fn get_interaction_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<Message>;

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
                f: F
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
                f: F
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

            pub async fn get_followup_message(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<Message>;

            pub async fn defer(&self, [ &self.bot ]) -> serenity::Result<()>;
        }
    }
}

impl AutocompleteCtx {
    delegate! {
        to self.interaction {
            pub async fn create_autocomplete_response<F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<()>
            where
                F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse;
        }
    }
}

impl MessageComponentCtx {
    delegate! {
        to self.interaction {
            pub async fn get_interaction_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<Message>;

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

            pub async fn get_followup_message(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<Message>;

            pub async fn delete_followup_message(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<()>;

            pub async fn defer(&self, [ &self.bot ]) -> serenity::Result<()>;
        }
    }
}

impl ModalCtx {
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

            pub async fn edit_original_interaction_response<F>(&self, [ &self.bot ], f: F) -> serenity::Result<Message>
            where
                F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse;

            pub async fn delete_original_interaction_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<()>;

            pub async fn create_followup_message<'a, F>(&self, [ &self.bot ], f: F) -> serenity::Result<Message>
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
