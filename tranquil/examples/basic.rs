use serenity::{
    Client,
    all::{CommandInteraction, GatewayIntents},
    prelude::Context,
};
use tokio::{select, signal};
use tranquil::{
    Tranquil,
    interaction::command::{Command, Run, RunError},
};

// #[derive(Command)]
// #[tranquil(
//     name("my-ping-command", de = "mein-ping-kommando"),
//     description(
//         "Check if the Bot is still alive",
//         de = "Schaue ob der Bot noch lebt",
//         en_US = "Check if the Bot is still alive",
//     )
// )]
// enum Ping {
//     Bot,
//     User {
//         #[tranquil(description("The user to ping"))]
//         user: String,
//     },
// }

#[derive(Command)]
#[tranquil(description("le ping"))]
struct Ping {}

impl Run for Ping {
    async fn run(self, ctx: Context, interaction: CommandInteraction) -> Result<(), RunError> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    // Load a .env file if one exists
    if let Err(error) = dotenvy::dotenv() {
        if !error.not_found() {
            panic!("{error}")
        }
    }

    // Log to console based on the RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    // Setup tranquil
    let tranquil = Tranquil::new();

    // Setup serenity using tranquil as a framework
    let mut client = Client::builder(
        std::env::var("DISCORD_TOKEN").unwrap(),
        GatewayIntents::empty(),
    )
    .framework(tranquil)
    .await
    .unwrap();

    // Run the Bot until it fully crashes or Ctrl+C is hit
    select! {
        result = client.start() => result.unwrap(),
        result = signal::ctrl_c() => result.unwrap(),
    }

    // Gracefully shutdown
    client.shard_manager.shutdown_all().await;
}
