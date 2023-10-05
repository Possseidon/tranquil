use anyhow::Result;
use tranquil::{
    context::command::CommandCtx,
    macros::{command_provider, slash},
    module::Module,
};

#[derive(Module)]
pub(crate) struct SubcommandModule;

async fn pong(ctx: CommandCtx) -> Result<()> {
    ctx.respond(|response| response.interaction_response_data(|data| data.content("Pong!")))
        .await?;
    Ok(())
}

#[command_provider]
impl SubcommandModule {
    #[slash]
    async fn member_add(&self, ctx: CommandCtx) -> Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn member_delete(&self, ctx: CommandCtx) -> Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn member_info_age(&self, ctx: CommandCtx) -> Result<()> {
        pong(ctx).await
    }

    #[slash(rename = "member info nick-name")]
    async fn member_info_nick_name(&self, ctx: CommandCtx) -> Result<()> {
        pong(ctx).await
    }
}
