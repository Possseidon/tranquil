use serenity::{
    builder::CreateButton,
    model::{application::component::ButtonStyle, channel::ReactionType},
};

use crate::{custom_id::custom_id_encode, interaction::Interact};

#[derive(Clone)]
pub struct Button {
    label: ButtonLabel,
    color: ButtonColor,
    enabled: bool,
}

#[derive(Clone)]
pub struct LinkButton {
    label: ButtonLabel,
    url: String,
    enabled: bool,
}

#[derive(Clone)]
pub enum ButtonLabel {
    Text { text: String },
    Emoji { emoji: ReactionType },
    EmojiWithText { emoji: ReactionType, text: String },
}

#[derive(Clone, Copy, Default)]
pub enum ButtonColor {
    #[default]
    Primary,
    Secondary,
    Success,
    Danger,
}

impl Button {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            label: ButtonLabel::text(text),
            color: ButtonColor::default(),
            enabled: true,
        }
    }

    pub fn emoji(emoji: impl Into<ReactionType>) -> Self {
        Self {
            label: ButtonLabel::emoji(emoji),
            color: ButtonColor::default(),
            enabled: true,
        }
    }

    pub fn emoji_with_text(emoji: impl Into<ReactionType>, text: impl Into<String>) -> Self {
        Self {
            label: ButtonLabel::emoji_with_text(emoji, text),
            color: ButtonColor::default(),
            enabled: true,
        }
    }

    pub fn color(self, color: ButtonColor) -> Self {
        Self { color, ..self }
    }

    pub fn secondary(self) -> Self {
        Self {
            color: ButtonColor::Secondary,
            ..self
        }
    }

    pub fn success(self) -> Self {
        Self {
            color: ButtonColor::Success,
            ..self
        }
    }

    pub fn danger(self) -> Self {
        Self {
            color: ButtonColor::Danger,
            ..self
        }
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

    fn create_with_custom_id(self, custom_id: String) -> CreateButton {
        let mut button = CreateButton::default();

        button.custom_id(custom_id);

        button.style(match self.color {
            ButtonColor::Primary => ButtonStyle::Primary,
            ButtonColor::Secondary => ButtonStyle::Secondary,
            ButtonColor::Success => ButtonStyle::Success,
            ButtonColor::Danger => ButtonStyle::Danger,
        });

        match self.label {
            ButtonLabel::Text { text } => {
                button.label(text);
            }
            ButtonLabel::Emoji { emoji } => {
                button.emoji(emoji);
            }
            ButtonLabel::EmojiWithText { emoji, text } => {
                button.emoji(emoji);
                button.label(text);
            }
        }

        button.disabled(!self.enabled);

        button
    }

    pub fn create<T: Interact>(self, interact: &T) -> CreateButton {
        self.create_with_custom_id(format!(
            "{} {}",
            T::UUID.simple(),
            custom_id_encode(interact),
        ))
    }
}

impl LinkButton {
    pub fn text(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: ButtonLabel::text(text),
            url: url.into(),
            enabled: true,
        }
    }

    pub fn emoji(emoji: impl Into<ReactionType>, url: impl Into<String>) -> Self {
        Self {
            label: ButtonLabel::emoji(emoji),
            url: url.into(),
            enabled: true,
        }
    }

    pub fn emoji_with_text(
        emoji: impl Into<ReactionType>,
        text: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            label: ButtonLabel::emoji_with_text(emoji, text),
            url: url.into(),
            enabled: true,
        }
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

    pub fn create(self) -> CreateButton {
        let mut button = CreateButton::default();

        button.url(self.url);
        button.style(ButtonStyle::Link);

        match self.label {
            ButtonLabel::Text { text } => {
                button.label(text);
            }
            ButtonLabel::Emoji { emoji } => {
                button.emoji(emoji);
            }
            ButtonLabel::EmojiWithText { emoji, text } => {
                button.emoji(emoji);
                button.label(text);
            }
        }

        button.disabled(!self.enabled);

        button
    }
}

impl ButtonLabel {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn emoji(emoji: impl Into<ReactionType>) -> Self {
        Self::Emoji {
            emoji: emoji.into(),
        }
    }

    pub fn emoji_with_text(emoji: impl Into<ReactionType>, text: impl Into<String>) -> Self {
        Self::EmojiWithText {
            emoji: emoji.into(),
            text: text.into(),
        }
    }
}
