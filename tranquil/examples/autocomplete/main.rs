use anyhow::Result;
use tranquil::{
    bot::Bot,
    utils::{debug_guilds_from_env, discord_token_from_env, dotenv_if_exists},
};

mod autocomplete_module;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv_if_exists()?;

    Bot::new()
        .application_command_update(debug_guilds_from_env()?)
        .register(autocomplete_module::AutocompleteModule)
        .run_until_ctrl_c(discord_token_from_env()?)
        .await
}
