use async_trait::async_trait;
use tranquil::{
    context::CommandCtx,
    l10n::{L10n, L10nLoadError},
    macros::{command_provider, slash},
    module::Module,
    resolve::Choices,
};

pub(crate) struct ExampleModule;

#[async_trait]
impl Module for ExampleModule {
    async fn l10n(&self) -> Result<L10n, L10nLoadError> {
        L10n::from_yaml_file("tranquil/examples/l10n/example_module_l10n.yaml").await
    }
}

async fn pong(ctx: CommandCtx) -> anyhow::Result<()> {
    ctx.create_response(|response| {
        response.interaction_response_data(|data| data.content("Pong!"))
    })
    .await?;
    Ok(())
}

#[command_provider]
#[allow(unused_variables)]
impl ExampleModule {
    #[slash]
    async fn members_add(&self, ctx: CommandCtx, member: String) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn members_color(
        &self,
        ctx: CommandCtx,
        member: String,
        color: Color,
    ) -> anyhow::Result<()> {
        pong(ctx).await
    }
}

#[derive(Choices)]
pub(crate) enum Color {
    Red,
    Green,
    Blue,
}
