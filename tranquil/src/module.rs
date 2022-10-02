use std::sync::Arc;

use serenity::{async_trait, model::gateway::GatewayIntents};

use crate::slash_command::SlashCommands;

#[async_trait]
pub trait Module: Send + Sync {
    fn intents(&self) -> GatewayIntents {
        GatewayIntents::empty()
    }

    fn slash_commands(self: Arc<Self>) -> SlashCommands {
        SlashCommands::default()
    }
}
