use anyhow::Result;
use async_trait::async_trait;
use serenity::model::gateway::GatewayIntents;
use uuid::Uuid;

use crate::{
    command::CommandProvider,
    context::{ComponentCtx, ModalCtx},
    l10n::{L10n, L10nLoadError},
};

pub use tranquil_macros::Module;

#[async_trait]
pub trait Module: CommandProvider + Send + Sync {
    fn intents(&self) -> GatewayIntents {
        GatewayIntents::empty()
    }

    async fn l10n(&self) -> Result<L10n, L10nLoadError> {
        Ok(L10n::new())
    }

    fn interaction_uuids(&self) -> &'static [Uuid] {
        &[]
    }

    async fn interact(&self, _uuid: Uuid, _state: &str, _ctx: ComponentCtx) -> Result<()> {
        panic!("module does not handle any interactions")
    }

    fn modal_uuids(&self) -> &'static [Uuid] {
        &[]
    }

    async fn submit(&self, _uuid: Uuid, _state: &str, _ctx: ModalCtx) -> Result<()> {
        panic!("module does not handle any modals")
    }
}
