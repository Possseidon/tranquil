use std::time::Duration;

use async_trait::async_trait;
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateActionRow, CreateButton};
use tokio::time::sleep;
use tranquil::{
    button::{Button, ButtonColor, LinkButton},
    context::{CommandCtx, ComponentCtx},
    handle_interactions,
    interaction::Interact,
    macros::{command_provider, slash},
    module::Module,
    select_menu::{
        MultiSelect, MultiSelectHandler, Select, SelectHandler, SelectMenuChoice, SelectMenuOption,
        SelectMenuOptions,
    },
};
use uuid::{uuid, Uuid};

pub(crate) struct InteractionModule;

impl Module for InteractionModule {
    handle_interactions![
        PingButton,
        CounterButton,
        SelectHandler<Color>,
        MultiSelectHandler<Color>,
    ];
}

#[command_provider]
impl InteractionModule {
    #[slash]
    async fn buttons_link(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| {
                data.components(|components| {
                    components.create_action_row(|row| row.add_button(youtube_button().create()))
                })
            })
        })
        .await?;

        Ok(())
    }

    #[slash]
    async fn buttons_ping(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| {
                data.components(|components| {
                    components.create_action_row(|row| row.add_button(PingButton::create()))
                })
            })
        })
        .await?;

        Ok(())
    }

    #[slash]
    async fn buttons_counter(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| {
                data.components(|components| {
                    components.set_action_row(CounterButton::create_row(5, true))
                })
            })
        })
        .await?;

        Ok(())
    }

    #[slash]
    async fn select_choice(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| {
                data.components(|components| {
                    components.create_action_row(|row| row.add_select_menu(Color::create()))
                })
            })
        })
        .await?;

        Ok(())
    }

    #[slash]
    async fn select_options(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| {
                data.components(|components| {
                    components.create_action_row(|row| row.add_select_menu(Color::create_multi()))
                })
            })
        })
        .await?;

        Ok(())
    }
}

fn youtube_button() -> LinkButton {
    LinkButton::emoji_with_text(
        '📼',
        "YouTube",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    )
}

#[derive(Serialize, Deserialize)]
struct PingButton;

impl PingButton {
    fn create() -> CreateButton {
        Button::text("Ping!")
            .color(ButtonColor::Success)
            .create(&Self)
    }
}

#[async_trait]
impl Interact for PingButton {
    const UUID: Uuid = uuid!("556fffe5-3849-43d0-b099-17ed170a7336");

    type Module = InteractionModule;

    async fn interact(self, _module: &Self::Module, ctx: ComponentCtx) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| data.content("Pong!"))
        })
        .await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct CounterButton {
    kind: ButtonAction,
    value: i32,
    enabled: bool,
}

#[derive(Serialize, Deserialize)]
enum ButtonAction {
    MinusTen,
    Decrement,
    Confirm,
    Increment,
    PlusTen,
}

impl CounterButton {
    fn create_row(value: i32, enabled: bool) -> CreateActionRow {
        let mut row = CreateActionRow::default();

        for (kind, value) in [
            (ButtonAction::MinusTen, value - 10),
            (ButtonAction::Decrement, value - 1),
            (ButtonAction::Confirm, value),
            (ButtonAction::Increment, value + 1),
            (ButtonAction::PlusTen, value + 10),
        ] {
            row.add_button(
                CounterButton {
                    kind,
                    value,
                    enabled,
                }
                .create(),
            );
        }

        row
    }

    fn create(&self) -> CreateButton {
        match self.kind {
            ButtonAction::MinusTen => Button::emoji('⏪'),
            ButtonAction::Decrement => Button::emoji('◀'),
            ButtonAction::Confirm => Button::text(format!("{}", self.value)).success(),
            ButtonAction::Increment => Button::emoji('▶'),
            ButtonAction::PlusTen => Button::emoji('⏩'),
        }
        .enabled(self.enabled)
        .create(self)
    }
}

#[async_trait]
impl Interact for CounterButton {
    const UUID: Uuid = uuid!("00269e42-1dd5-4d28-920b-d583d277f042");

    type Module = InteractionModule;

    async fn interact(self, _module: &Self::Module, ctx: ComponentCtx) -> anyhow::Result<()> {
        match self.kind {
            ButtonAction::MinusTen
            | ButtonAction::Decrement
            | ButtonAction::Increment
            | ButtonAction::PlusTen => {
                ctx.defer().await?;
                ctx.edit_response(|response| {
                    response.components(|components| {
                        components.set_action_row(Self::create_row(self.value, self.enabled))
                    })
                })
                .await?;
            }
            ButtonAction::Confirm => {
                ctx.defer().await?;

                let mut message = ctx.interaction.message;

                message
                    .edit(&ctx.bot, |edit| {
                        edit.components(|components| {
                            components.set_action_row(Self::create_row(self.value, false))
                        })
                    })
                    .await?;

                for i in (1..=self.value).rev() {
                    message.edit(&ctx.bot, |edit| edit.content(i)).await?;

                    sleep(Duration::from_secs(1)).await;
                }

                message
                    .edit(&ctx.bot, |edit| {
                        edit.content("Done!").components(|components| {
                            components.set_action_row(Self::create_row(self.value, true))
                        })
                    })
                    .await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, SelectMenuChoice, SelectMenuOptions, EnumSetType)]
#[module(InteractionModule)]
#[choice_uuid("a1a0eff0-1031-42a9-a7b8-f44a7e7006dc")]
#[options_uuid("db48bcce-45de-4e59-92c6-ee9e696d5373")]
#[options_count(3)]
enum Color {
    #[option('🔴', "Red")]
    Red,
    #[option('🟠', "Orange")]
    Orange,
    #[option('🟡', "Yellow")]
    Yellow,
    #[option('🟢', "Green", default)]
    Green,
    #[option('🔵', "Blue", "I'm blue dabedi dabedei")]
    Blue,
    #[option('🟣', "Purple")]
    Purple,
}

#[async_trait]
impl Select for Color {
    async fn select(self, _module: &Self::Module, mut ctx: ComponentCtx) -> anyhow::Result<()> {
        ctx.defer().await?;
        ctx.interaction
            .message
            .edit(&ctx.bot, |edit| {
                edit.content(format!("**You selected:**\n```rust\n{:?}\n```", self))
            })
            .await?;

        Ok(())
    }
}

#[async_trait]
impl MultiSelect for Color {
    async fn multi_select(
        values: EnumSet<Self>,
        _module: &Self::Module,
        mut ctx: ComponentCtx,
    ) -> anyhow::Result<()> {
        ctx.defer().await?;
        ctx.interaction
            .message
            .edit(&ctx.bot, |edit| {
                edit.content(format!("**You selected:**\n```rust\n{:?}\n```", values))
            })
            .await?;

        Ok(())
    }
}
