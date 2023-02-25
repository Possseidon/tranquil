use tranquil::{
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{command_provider, slash},
    module::Module,
};

pub(crate) struct PingModule;

impl Module for PingModule {}

impl CommandL10nProvider for PingModule {}

#[command_provider]
impl PingModule {
    #[slash]
    async fn ping(&self, ctx: CommandContext) -> anyhow::Result<()> {
        ctx.create_response(|response| {
            response.interaction_response_data(|data| data.content("Pong!"))
        })
        .await?;
        Ok(())
    }
}
