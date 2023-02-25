use async_trait::async_trait;
use tranquil::{
    command::CommandContext,
    l10n::{CommandL10nProvider, L10n, L10nLoadError},
    macros::{command_provider, slash, Choices},
    module::Module,
};

pub(crate) struct ExampleModule;

impl Module for ExampleModule {}

#[async_trait]
impl CommandL10nProvider for ExampleModule {
    async fn l10n(&self) -> Result<L10n, L10nLoadError> {
        L10n::from_yaml_file("tranquil/examples/l10n/example_module_l10n.yaml").await
    }
}

async fn pong(ctx: CommandContext) -> anyhow::Result<()> {
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
    async fn members_add(&self, ctx: CommandContext, member: String) -> anyhow::Result<()> {
        pong(ctx).await
    }

    #[slash]
    async fn members_color(
        &self,
        ctx: CommandContext,
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
