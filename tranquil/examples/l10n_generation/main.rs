#[path = "../l10n/example_module.rs"]
mod example_module;

use std::sync::Arc;

use tranquil::{
    command::CommandProvider,
    l10n::{L10n, Locale},
    AnyResult,
};

fn main() -> AnyResult<()> {
    let module = Arc::new(example_module::ExampleModule);
    let command_map = module.command_map()?;

    let locales = Locale::German | Locale::EnglishUS;

    // Generate stubs for all commands inside a module.
    let mut stubs = L10n::command_stubs(&command_map, locales)?;

    // Generate stubs for a Choices enum.
    let choice_stubs = L10n::choice_stubs::<example_module::Color>(locales);

    // Merge different l10n together.
    stubs.merge(choice_stubs)?;

    // Print it out as yaml.
    print!("{}", stubs.to_yaml()?);

    Ok(())
}
