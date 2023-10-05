use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use serenity::{
    builder::{CreateActionRow, CreateComponents, CreateInteractionResponse},
    model::application::{component::InputTextStyle, interaction::InteractionResponseType},
};
use uuid::Uuid;

use crate::{
    context::{command::CommandCtx, component::ComponentCtx},
    custom_id::custom_id_encode,
};

#[async_trait]
pub trait OpenModal: Serialize {
    const UUID: Uuid;

    type Module;
    type Response: ModalResponse;

    async fn submit(&self, module: &Self::Module, response: Self::Response) -> Result<()>;
}

pub trait ModalResponse {}

#[derive(Clone)]
pub struct Modal {
    pub title: String,
    pub text_inputs: Vec<TextInput>,
}

#[derive(Clone)]
pub struct TextInput {
    pub style: InputTextStyle,
    pub label: String,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub required: bool,
    pub value: String,
    pub placeholder: String,
}

impl Modal {
    pub fn new(title: impl Into<String>, text_inputs: impl IntoIterator<Item = TextInput>) -> Self {
        Self {
            title: title.into(),
            text_inputs: text_inputs.into_iter().collect(),
        }
    }

    pub async fn open<T: OpenModal>(
        self,
        ctx: &impl RespondCtx,
        submit: &T,
    ) -> serenity::Result<()> {
        let custom_id = format!("{} {}", T::UUID.simple(), custom_id_encode(submit));

        let components = CreateComponents(
            self.text_inputs
                .into_iter()
                .enumerate()
                .map(|(i, text_input)| {
                    let mut row = CreateActionRow::default();

                    row.create_input_text(|input| {
                        input
                            .custom_id(i)
                            .style(text_input.style)
                            .label(text_input.label)
                            .required(text_input.required)
                            .value(text_input.value)
                            .placeholder(text_input.placeholder);

                        if let Some(length) = text_input.min_length {
                            input.min_length(length);
                        }
                        if let Some(length) = text_input.max_length {
                            input.max_length(length);
                        }

                        input
                    });

                    row.build()
                })
                .collect(),
        );

        ctx.create_response(move |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(move |data| {
                    data.custom_id(custom_id)
                        .title(self.title)
                        .set_components(components)
                })
        })
        .await
    }
}

impl TextInput {
    pub fn new(style: InputTextStyle, label: impl Into<String>) -> Self {
        Self {
            style,
            label: label.into(),
            min_length: None,
            max_length: None,
            required: true,
            value: String::new(),
            placeholder: String::new(),
        }
    }

    pub fn short(label: impl Into<String>) -> Self {
        Self::new(InputTextStyle::Short, label)
    }

    pub fn paragraph(label: impl Into<String>) -> Self {
        Self::new(InputTextStyle::Paragraph, label)
    }

    pub fn min_length(self, length: u64) -> Self {
        Self {
            min_length: Some(length),
            ..self
        }
    }

    pub fn max_length(self, length: u64) -> Self {
        Self {
            max_length: Some(length),
            ..self
        }
    }

    pub fn required(self, required: bool) -> Self {
        Self { required, ..self }
    }

    pub fn optional(self) -> Self {
        Self {
            required: false,
            ..self
        }
    }

    pub fn value(self, value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            ..self
        }
    }

    pub fn placeholder(self, placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..self
        }
    }
}

#[async_trait]
pub trait RespondCtx {
    async fn create_response<'a, F>(&self, f: F) -> serenity::Result<()>
    where
        for<'b> F: FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>
            + Send;
}

#[async_trait]
impl RespondCtx for CommandCtx {
    async fn create_response<'a, F>(&self, f: F) -> serenity::Result<()>
    where
        for<'b> F: FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>
            + Send,
    {
        self.interaction
            .create_interaction_response(&self.bot, f)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl RespondCtx for ComponentCtx {
    async fn create_response<'a, F>(&self, f: F) -> serenity::Result<()>
    where
        for<'b> F: FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>
            + Send,
    {
        self.interaction
            .create_interaction_response(&self.bot, f)
            .await?;
        Ok(())
    }
}
