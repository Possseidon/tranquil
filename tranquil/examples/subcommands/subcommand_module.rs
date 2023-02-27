use tranquil::{
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{command_provider, slash},
    module::Module,
};

#[derive(Module, CommandL10nProvider)]
pub(crate) struct SubcommandModule;

async fn pong(ctx: CommandContext) -> anyhow::Result<()> {
    ctx.create_response(|response| {
        response.interaction_response_data(|data| data.content("Pong!"))
    })
    .await?;
    Ok(())
}

#[command_provider]
impl SubcommandModule {
    #[slash]
    async fn member_add(&self, ctx: CommandContext) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn member_delete(&self, ctx: CommandContext) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn member_info_age(&self, ctx: CommandContext) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash(rename = "member info nick-name")]
    async fn member_info_nick_name(&self, ctx: CommandContext) -> anyhow::Result<()> {
        pong(ctx).await
    }
}
