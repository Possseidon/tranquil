use serenity::all::{Context, CreateActionRow, CreateModal, ModalInteraction};

use super::CustomIdAssignment;

pub trait ModalBuilder<M: Modal> {
    fn build(self) -> CreateModal {
        CreateModal::new(custom_id, title).components(components)
    }
}

pub trait Modal {
    const CUSTOM_ID: CustomIdAssignment = CustomIdAssignment::OnStart;

    fn resolve(interaction: &mut ModalInteraction) -> Self;
}

pub trait Submit<M: Modal> {
    fn submit(
        &self,
        modal: M,
        ctx: Context,
        interaction: ModalInteraction,
    ) -> impl Future<Output = Result<()>> + Send;
}

struct CreateModal2 {
    title: String,
    rows: Vec<CreateActionRow>,
}
