use anyhow::Result;
use indoc::indoc;
use tranquil::{
    autocomplete::{Autocomplete, Focusable},
    context::{AutocompleteCtx, CommandCtx},
    macros::{autocompleter, command_provider, slash},
    module::Module,
};

#[derive(Module)]
pub(crate) struct AutocompleteModule;

impl AutocompleteModule {
    #[autocompleter]
    async fn autocomplete_echo_simple(&self, ctx: AutocompleteCtx, value: String) -> Result<()> {
        let you_typed = format!("You typed: {value}");

        ctx.create_response(|response| response.add_string_choice(you_typed, value))
            .await?;

        Ok(())
    }

    #[autocompleter]
    async fn my_complex_autocompleter(
        &self,
        ctx: AutocompleteCtx,
        not_autocompleted: Option<String>,
        autocompleted: Focusable<Option<String>>,
        optional: Option<String>,
        optional_autocompleted: Option<String>,
    ) -> Result<()> {
        let not_autocompleted_completion = format!("not_autocompleted: {not_autocompleted:?}");
        let autocompleted_completion = format!("autocompleted: {autocompleted:?}");
        let optional_completion = format!("optional: {optional:?}");
        let optional_autocompleted_completion =
            format!("optional_autocompleted: {optional_autocompleted:?}");

        ctx.create_response(|response| {
            response
                .add_string_choice(
                    not_autocompleted_completion,
                    not_autocompleted.unwrap_or_else(|| "empty".to_string()),
                )
                .add_string_choice(
                    autocompleted_completion,
                    autocompleted.current.unwrap_or_else(|| "empty".to_string()),
                )
                .add_string_choice(
                    optional_completion,
                    optional.unwrap_or_else(|| "empty".to_string()),
                )
                .add_string_choice(
                    optional_autocompleted_completion,
                    optional_autocompleted.unwrap_or_else(|| "empty".to_string()),
                )
        })
        .await?;
        Ok(())
    }
}

#[command_provider]
impl AutocompleteModule {
    #[slash(autocomplete)]
    async fn echo_simple(&self, ctx: CommandCtx, value: Autocomplete<String>) -> Result<()> {
        ctx.create_response(|response| {
            response
                .interaction_response_data(|data| data.content(format!("```rust\n{value:?}\n```")))
        })
        .await?;
        Ok(())
    }

    #[slash(autocomplete = "my_complex_autocompleter")]
    async fn echo_complex(
        &self,
        ctx: CommandCtx,
        not_autocompleted: String,
        autocompleted: Autocomplete<String>,
        optional: Option<String>,
        optional_autocompleted: Autocomplete<Option<String>>,
    ) -> Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| {
                data.content(format!(
                    indoc! {r#"
                    ```rust
                    not_autocompleted:      {:?}
                    autocompleted:          {:?}
                    optional:               {:?}
                    optional_autocompleted: {:?}
                    ```
                    "#},
                    not_autocompleted, autocompleted, optional, optional_autocompleted
                ))
            })
        })
        .await?;
        Ok(())
    }
}
