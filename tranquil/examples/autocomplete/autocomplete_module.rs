use indoc::indoc;
use tranquil::{
    autocomplete::{Autocomplete, AutocompleteContext, Focusable},
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{autocompleter, command_provider, slash},
    module::Module,
    AnyResult,
};

pub(crate) struct AutocompleteModule;

impl Module for AutocompleteModule {}

impl CommandL10nProvider for AutocompleteModule {}

impl AutocompleteModule {
    #[autocompleter]
    async fn autocomplete_echo_simple(
        &self,
        ctx: AutocompleteContext,
        value: String,
    ) -> AnyResult<()> {
        let you_typed = format!("You typed: {value}");

        ctx.create_response(|response| response.add_string_choice(you_typed, value))
            .await?;
        Ok(())
    }

    #[autocompleter]
    async fn my_complex_autocompleter(
        &self,
        ctx: AutocompleteContext,
        not_autocompleted: Option<String>,
        autocompleted: Focusable<Option<String>>,
        optional: Option<String>,
        optional_autocompleted: Option<String>,
    ) -> AnyResult<()> {
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
    async fn echo_simple(&self, ctx: CommandContext, value: Autocomplete<String>) -> AnyResult<()> {
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
        ctx: CommandContext,
        not_autocompleted: String,
        autocompleted: Autocomplete<String>,
        optional: Option<String>,
        optional_autocompleted: Autocomplete<Option<String>>,
    ) -> AnyResult<()> {
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
