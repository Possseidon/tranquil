use tranquil::{
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{command_provider, slash, Choices},
    module::Module,
    AnyResult,
};

pub(crate) struct ExampleModule;

impl Module for ExampleModule {}

impl CommandL10nProvider for ExampleModule {
    fn l10n_path(&self) -> Option<&str> {
        Some("tranquil/examples/l10n/example_module_l10n.yaml")
    }
}

async fn pong(ctx: CommandContext) -> AnyResult<()> {
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
    async fn members_add(&self, ctx: CommandContext, member: String) -> AnyResult<()> {
        pong(ctx).await
    }

    #[slash]
    async fn members_color(
        &self,
        ctx: CommandContext,
        member: String,
        color: Color,
    ) -> AnyResult<()> {
        pong(ctx).await
    }
}

#[derive(Choices)]
pub(crate) enum Color {
    Red,
    Green,
    Blue,
}
