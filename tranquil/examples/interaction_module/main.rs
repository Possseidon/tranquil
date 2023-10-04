use tranquil::{
    bot::Bot,
    utils::{debug_guilds_from_env, discord_token_from_env, dotenv_if_exists},
};

mod interaction_module;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv_if_exists()?;

    Bot::new()
        .application_command_update(debug_guilds_from_env()?)
        .register(interaction_module::InteractionModule)
        .run_until_ctrl_c(discord_token_from_env()?)
        .await
}