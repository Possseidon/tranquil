use serenity::model::gateway::GatewayIntents;

use crate::{command::CommandProvider, l10n::CommandL10nProvider};

pub trait Module: CommandProvider + CommandL10nProvider + Send + Sync {
    fn intents(&self) -> GatewayIntents {
        GatewayIntents::empty()
    }
}
