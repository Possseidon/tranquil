use std::sync::Arc;

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
            modal::ModalSubmitInteraction, InteractionResponseType,
        },
        channel::Message,
        id::MessageId,
    },
};

pub struct CommandCtx {
    pub bot: Context,
    pub interaction: Arc<ApplicationCommandInteraction>,
}

impl CommandCtx {
    pub async fn create_response<'a, F>(self, f: F) -> serenity::Result<RespondedCommandCtx>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        self.interaction
            .create_interaction_response(&self.bot, f)
            .await?;
        Ok(self.responded())
    }

    pub async fn defer(self) -> serenity::Result<RespondedCommandCtx> {
        self.interaction.defer(&self.bot).await?;
        Ok(self.responded())
    }

    pub async fn defer_ephemeral(self) -> serenity::Result<RespondedCommandCtx> {
        self.create_response(|response| {
            response
                .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|data| data.ephemeral(true))
        })
        .await
    }

    fn responded(self) -> RespondedCommandCtx {
        RespondedCommandCtx {
            bot: self.bot,
            interaction: self.interaction,
        }
    }
}

pub struct RespondedCommandCtx {
    pub bot: Context,
    pub interaction: Arc<ApplicationCommandInteraction>,
}

impl RespondedCommandCtx {
    delegate! {
        to self.interaction {
            #[call(get_interaction_response)]
            pub async fn get_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<Message>;

            #[call(edit_original_interaction_response)]
            pub async fn edit_response<F>(
                &self,
                [ &self.bot ],
                f: F
            ) -> serenity::Result<Message>
            where
                F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse;

            #[call(delete_original_interaction_response)]
            pub async fn delete_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<()>;

            #[call(create_followup_message)]
            pub async fn create_followup<'a, F>(
                &self,
                [ &self.bot ],
                f: F
            ) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            #[call(edit_followup_message)]
            pub async fn edit_followup<'a, F>(
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

            #[call(get_followup_message)]
            pub async fn get_followup(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<Message>;
        }
    }
}

// TODO: State-Machine-ify

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

pub type AutocompleteCtx = Ctx<AutocompleteInteraction>;
pub type ComponentCtx = Ctx<MessageComponentInteraction>;
pub type ModalCtx = Ctx<ModalSubmitInteraction>;

impl AutocompleteCtx {
    delegate! {
        to self.interaction {
            #[call(create_autocomplete_response)]
            pub async fn create_response<F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<()>
            where
                F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse;
        }
    }
}

impl ComponentCtx {
    delegate! {
        to self.interaction {
            #[call(get_interaction_response)]
            pub async fn get_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<Message>;

            #[call(create_interaction_response)]
            pub async fn create_response<'a, F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<()>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponse<'a>,
                ) -> &'b mut CreateInteractionResponse<'a>;

            #[call(edit_original_interaction_response)]
            pub async fn edit_response<F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<Message>
            where
                F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse;

            #[call(delete_original_interaction_response)]
            pub async fn delete_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<()>;

            #[call(create_followup_message)]
            pub async fn create_followup<'a, F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            #[call(edit_followup_message)]
            pub async fn edit_followup<'a, F>(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
                f: F,
            ) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            #[call(get_followup_message)]
            pub async fn get_followup(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<Message>;

            #[call(delete_followup_message)]
            pub async fn delete_followup(
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
            #[call(get_interaction_response)]
            pub async fn get_response(&self, [ &self.bot ]) -> serenity::Result<Message>;

            #[call(create_interaction_response)]
            pub async fn create_response<'a, F>(
                &self,
                [ &self.bot ],
                f: F,
            ) -> serenity::Result<()>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponse<'a>,
                ) -> &'b mut CreateInteractionResponse<'a>;

            #[call(edit_original_interaction_response)]
            pub async fn edit_response<F>(&self, [ &self.bot ], f: F) -> serenity::Result<Message>
            where
                F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse;

            #[call(delete_original_interaction_response)]
            pub async fn delete_response(
                &self,
                [ &self.bot ],
            ) -> serenity::Result<()>;

            #[call(create_followup_message)]
            pub async fn create_followup<'a, F>(&self, [ &self.bot ], f: F) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            #[call(edit_followup_message)]
            pub async fn edit_followup<'a, F>(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
                f: F,
            ) -> serenity::Result<Message>
            where
                for<'b> F: FnOnce(
                    &'b mut CreateInteractionResponseFollowup<'a>,
                ) -> &'b mut CreateInteractionResponseFollowup<'a>;

            #[call(delete_followup_message)]
            pub async fn delete_followup(
                &self,
                [ &self.bot ],
                message_id: impl Into<MessageId>,
            ) -> serenity::Result<()>;

            pub async fn defer(&self, [ &self.bot ]) -> serenity::Result<()>;
        }
    }
}
