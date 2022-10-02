use serenity::model::gateway::GatewayIntents;

use crate::slash_command::SlashCommandProvider;

pub trait Module: SlashCommandProvider + Send + Sync {
    fn intents(&self) -> GatewayIntents {
        GatewayIntents::empty()
    }
}
