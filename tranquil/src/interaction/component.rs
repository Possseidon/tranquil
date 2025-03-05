use anyhow::Result;
use serenity::all::{ComponentInteraction, Context};

use super::CustomIdAssignment;

pub trait Component: Interact {
    fn custom_id() -> CustomIdAssignment {
        CustomIdAssignment::OnStart
    }

    fn resolve(interaction: &mut ComponentInteraction) -> Self;
}

pub trait Interact {
    fn interact(
        self,
        ctx: Context,
        interaction: ComponentInteraction,
    ) -> impl Future<Output = Result<()>> + Send;
}
