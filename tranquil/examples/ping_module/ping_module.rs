use anyhow::Result;
use tranquil::{
    context::command::CommandCtx,
    macros::{command_provider, slash},
    module::Module,
};

#[derive(Module)]
pub(crate) struct PingModule;

#[command_provider]
impl PingModule {
    #[slash]
    async fn ping(&self, ctx: CommandCtx) -> Result<()> {
        ctx.respond(|response| response.interaction_response_data(|data| data.content("Pong!")))
            .await?;
        Ok(())
    }
}
