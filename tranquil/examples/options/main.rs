use tranquil::{
    bot::Bot,
    utils::{debug_guilds_from_env, discord_token_from_env, dotenv_if_exists},
    AnyResult,
};

mod echo_module;

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv_if_exists()?;

    Bot::new()
        .application_command_update(debug_guilds_from_env()?)
        .register(echo_module::EchoModule)?
        .run_until_ctrl_c(discord_token_from_env()?)
        .await
}
