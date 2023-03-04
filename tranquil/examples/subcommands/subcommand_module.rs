use tranquil::{
    context::CommandCtx,
    macros::{command_provider, slash},
    module::Module,
};

#[derive(Module)]
pub(crate) struct SubcommandModule;

async fn pong(ctx: CommandCtx) -> anyhow::Result<()> {
    ctx.create_interaction_response(|response| {
        response.interaction_response_data(|data| data.content("Pong!"))
    })
    .await?;
    Ok(())
}

#[command_provider]
impl SubcommandModule {
    #[slash]
    async fn member_add(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn member_delete(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn member_info_age(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash(rename = "member info nick-name")]
    async fn member_info_nick_name(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        pong(ctx).await
    }
}
