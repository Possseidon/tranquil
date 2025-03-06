use serenity::{
    Client,
    all::{CommandInteraction, GatewayIntents, UserId},
    prelude::Context,
};
use tokio::{select, signal};
use tranquil::{
    Tranquil,
    interaction::{
        command::{Command, Run},
        error::Result,
    },
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

// #[derive(Command)]
// #[tranquil(description("le permission control"))]
// enum PermissionControl {
//     #[tranquil(
//         name(de = "neu-hinzufügen"),
//         description(
//             "Add a new permission to a user",
//             de = "Einem Benutzer eine neue Berechtigung hinzufügen",
//             en_US = "Interesting",
//         )
//     )]
//     AddNew {
//         #[tranquil(description("crazy user"))]
//         user: UserId,
//         #[tranquil(description("crazy permission"))]
//         permission: String,
//     },
//     #[tranquil(description("remove an old permission"))]
//     RemoveOld,
// }

/// `füge-mitglieder-hinzu-oder`
/// bli blub
///
/// - `en_GB` `colour` Add a member to a colour and yeah this is actually a really long description
///   that I am going to write. I want it to be even longer just so that I can get some more
///   wrapping
/// - `de` `mitglied` Füge Mitglieder hinzu oder entferne sie
/// - `fr` `francais` Je ne parle pas français
/// - `jp` `色` ユーザーの色
#[derive(Command)]
enum ColorCrazy {
    /// `get awesome-user` Get the color of a user
    GetAwesomeUser { user: UserId },
    /// `get-awesome user` Get the color of a user
    GetAwesomeUser2 { user: UserId },
    /// Set the color of a user
    Set { user: UserId, color: String },
    /// Clear the color of a user
    Clear { user: UserId },
}

impl Run for ColorCrazy {
    async fn run(self, ctx: Context, interaction: CommandInteraction) -> Result<()> {
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
