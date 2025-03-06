use serenity::all::{Context, ModalInteraction};

use super::CustomIdAssignment;

pub trait Modal: Submit {
    /// It rarely makes sense to use a fixed `custom_id` for something as short-lived as a modal.
    fn custom_id() -> CustomIdAssignment {
        CustomIdAssignment::OnStart
    }

    fn resolve(interaction: &mut ModalInteraction) -> Self;
}

pub trait Submit {
    fn submit(
        self,
        ctx: Context,
        interaction: ModalInteraction,
    ) -> impl Future<Output = Result<()>> + Send;
}
