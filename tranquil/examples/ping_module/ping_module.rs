use anyhow::bail;
use tranquil::{
    context::CommandCtx,
    macros::{command_provider, slash},
    module::Module,
};

#[derive(Module)]
pub(crate) struct PingModule;

#[command_provider]
impl PingModule {
    #[slash]
    async fn ping(&self, ctx: CommandCtx) -> anyhow::Result<()> {
        let ctx = ctx.defer().await?;
        bail!("oh no!")

        // ctx.create_response(|response| {
        //     response.interaction_response_data(|data| data.content("Pong!"))
        // })
        // .await?;

        // ctx.defer().await?;
        // ctx.delete_response().await?;

        // ctx.create_response(|response| response.kind(serenity::model::prelude::interaction::InteractionResponseType::DeferredChannelMessageWithSource). interaction_response_data(|data| data.ephemeral(true))).await?;

        // ^ works
        // ctx.create_followup(|followup| followup.content("followup 1"))
        //     .await?;

        // ctx.delete_followup_message(message_id);
        // ctx.delete_response().await?;
        // ctx.edit_followup(message_id, f);
        // ctx.edit_response(f);
        // ctx.get_followup(message_id);
        // let msg = ctx.get_response().await?;
        // println!("{msg:?}");

        // v doesn't work
    }
}

// if deferred, the first followup turns into the real response

// Initially all you can do is
// - create_response
// - defer / defer_ephemeral

// After that you can call pretty much all other functions except createing another initial response
// or deferring again (even after deleting the original)

// TODO: Add DeferredCtx and DeferredEphemeralCtx type that automatically gets deferred, giving immediate access to the right functions.
//       Do not make this the only way to defer, as it might need to decide whether to defer programmatically
