use tranquil::{
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{command_provider, slash},
    module::Module,
};

#[derive(Module, CommandL10nProvider)]
pub(crate) struct PingModule;

#[command_provider]
impl PingModule {
    #[slash]
    async fn ping(&self, ctx: CommandContext) -> anyhow::Result<()> {
        ctx.create_interaction_response(|response| {
            response.interaction_response_data(|data| data.content("Pong!"))
        })
        .await?;
        Ok(())
    }
}
