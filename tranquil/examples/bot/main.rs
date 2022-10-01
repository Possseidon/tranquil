use serenity::model::id::GuildId;
use tranquil::{
    bot::{ApplicationCommandUpdate, Bot},
    AnyResult,
};

fn env_var(token: &str) -> Result<String, std::env::VarError> {
    std::env::var(token).map_err(|err| {
        eprintln!("{err}: {token}");
        err
    })
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv::dotenv().ok();

    let bot = Bot::new()
        .application_command_update(
            std::env::var("DEBUG_GUILDS")
                .map(|var| {
                    Some(ApplicationCommandUpdate::Only(
                        var.split(',')
                            .map(|guild| GuildId(guild.trim().parse::<u64>().unwrap()))
                            .collect(),
                    ))
                })
                .unwrap_or_else(|_| Some(ApplicationCommandUpdate::default())),
        )
        // TODO: .register(MyModule)
        .run(env_var("DISCORD_TOKEN")?);

    tokio::select! {
        result = bot => if let Err(error) = result {
            eprintln!("{error}");
            Err("runtime error")?
        },
        result = tokio::signal::ctrl_c() => result?,
    }

    Ok(())
}
