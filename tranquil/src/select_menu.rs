use std::marker::PhantomData;

use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
use serenity::{builder::CreateSelectMenu, model::channel::ReactionType};
use uuid::Uuid;

use crate::{
    context::component::ComponentCtx, custom_id::custom_id_encode, interaction::Interact,
    module::Module,
};

pub struct SelectMenu {
    pub options: Vec<SelectMenuOption>,
    pub placeholder: String,
    pub min_values: u64,
    pub max_values: u64,
    pub enabled: bool,
}

pub struct SelectMenuOption {
    pub emoji: Option<ReactionType>,
    pub label: String,
    pub description: String,
    pub default: bool,
}

impl SelectMenu {
    pub fn new(options: impl IntoIterator<Item = SelectMenuOption>) -> Self {
        Self {
            options: options.into_iter().collect(),
            placeholder: String::new(),
            min_values: 1,
            max_values: 1,
            enabled: true,
        }
    }

    pub fn placeholder(self, placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..self
        }
    }

    pub fn min_values(self, min_values: u64) -> Self {
        Self { min_values, ..self }
    }

    pub fn max_values(self, max_values: u64) -> Self {
        Self { max_values, ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    pub fn disabled(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }

    fn create_with_custom_id(self, custom_id: String) -> CreateSelectMenu {
        let mut select_menu = CreateSelectMenu::default();

        select_menu.custom_id(custom_id);

        select_menu.options(|options| {
            for (i, option) in self.options.into_iter().enumerate() {
                options.create_option(|create_option| {
                    create_option
                        .label(option.label.clone())
                        .value(i)
                        .description(option.description)
                        .default_selection(option.default);

                    if let Some(emoji) = option.emoji {
                        create_option.emoji(emoji);
                    }

                    create_option
                });
            }

            options
        });

        select_menu.placeholder(self.placeholder);
        select_menu.min_values(self.min_values);
        select_menu.max_values(self.max_values);

        select_menu.disabled(!self.enabled);

        select_menu
    }

    pub fn create<T: Interact>(self, interact: &T) -> CreateSelectMenu {
        self.create_with_custom_id(format!(
            "{} {}",
            T::UUID.simple(),
            custom_id_encode(interact)
        ))
    }
}

impl SelectMenuOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: String::new(),
            emoji: None,
            default: false,
        }
    }

    pub fn with_emoji(emoji: impl Into<ReactionType>, label: impl Into<String>) -> Self {
        Self::new(label).emoji(emoji)
    }

    pub fn description(self, description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            ..self
        }
    }

    pub fn emoji(self, emoji: impl Into<ReactionType>) -> Self {
        Self {
            emoji: Some(emoji.into()),
            ..self
        }
    }

    pub fn default_if(self, default: bool) -> Self {
        Self { default, ..self }
    }

    pub fn default_selection(self) -> Self {
        self.default_if(true)
    }
}

pub use tranquil_macros::SelectMenuChoice;

pub trait SelectMenuChoice: Sized + Send + 'static {
    type Module: Module;

    const UUID: Uuid;
    const CHOICES: &'static [Self];
    const DEFAULT_CHOICE: Option<Self>;

    fn create() -> CreateSelectMenu;

    fn from_value(ctx: &str) -> Result<Self>;
}

#[derive(Serialize, Deserialize)]
pub struct SelectHandler<T: SelectMenuChoice>(PhantomData<&'static T>);

impl<T: SelectMenuChoice> Default for SelectHandler<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[async_trait]
impl<T: Select + Sync> Interact for SelectHandler<T> {
    const UUID: Uuid = T::UUID;

    type Module = T::Module;

    async fn interact(self, module: &Self::Module, ctx: ComponentCtx) -> Result<()> {
        let values = &ctx.interaction.data.values;

        if values.len() != 1 {
            bail!("only a single value expected")
        }

        T::from_value(&values[0])?.select(module, ctx).await
    }
}

#[async_trait]
pub trait Select: SelectMenuChoice {
    async fn select(self, module: &Self::Module, ctx: ComponentCtx) -> Result<()>;
}

pub use tranquil_macros::SelectMenuOptions;

pub trait SelectMenuOptions: EnumSetType + Sized + Send + 'static {
    type Module: Module;

    const UUID: Uuid;
    const DEFAULT_OPTION: Option<Self>;

    fn create_multi() -> CreateSelectMenu;

    fn from_values(ctx: &[String]) -> Result<EnumSet<Self>>;
}

#[derive(Serialize, Deserialize)]
pub struct MultiSelectHandler<T: SelectMenuOptions>(PhantomData<&'static T>);

impl<T: SelectMenuOptions> Default for MultiSelectHandler<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[async_trait]
impl<T: MultiSelect + Sync> Interact for MultiSelectHandler<T>
where
    EnumSet<T>: Send,
{
    const UUID: Uuid = T::UUID;

    type Module = T::Module;

    async fn interact(self, module: &Self::Module, ctx: ComponentCtx) -> Result<()> {
        T::multi_select(T::from_values(&ctx.interaction.data.values)?, module, ctx).await
    }
}

#[async_trait]
pub trait MultiSelect: SelectMenuOptions {
    async fn multi_select(
        values: EnumSet<Self>,
        module: &Self::Module,
        ctx: ComponentCtx,
    ) -> Result<()>;
}
