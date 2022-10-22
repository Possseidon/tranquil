use tranquil::{
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{command_provider, slash, Choices},
    module::Module,
    AnyResult,
};

pub(crate) struct ExampleModule;

impl Module for ExampleModule {}

impl CommandL10nProvider for ExampleModule {}

#[command_provider]
#[allow(unused_variables)]
impl ExampleModule {
    #[slash]
    async fn members_add(&self, ctx: CommandContext, member: String) -> AnyResult<()> {
        Ok(())
    }

    #[slash]
    async fn members_kick(
        &self,
        ctx: CommandContext,
        member: String,
        reason: String,
    ) -> AnyResult<()> {
        Ok(())
    }
}

#[derive(Choices)]
pub(crate) enum Color {
    Red,
    Green,
    Blue,
}
