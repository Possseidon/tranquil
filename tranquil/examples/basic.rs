use serenity::{
    Client,
    all::{CommandInteraction, GatewayIntents, UserId},
    prelude::Context,
};
use tokio::{select, signal};
use tranquil::{
    Tranquil,
    interaction::{
        command::{Command, Group, Run},
        error::Result,
    },
};

#[derive(Command)]
enum ColorCrazy {
    /// description
    Get(Group),
    GetUser(ColorGet),
    /// `get awesome-user` Get the color of a user
    GetAwesomeUser {
        user: UserId,
    },
    /// `get-awesome user` Get the color of a user
    GetAwesomeUser2 {
        user: UserId,
    },
    /// Set the color of a user
    Set {
        user: UserId,
        #[tranquil(autocomplete)]
        color: String,
    },
    /// Clear the color of a user
    Clear {
        user: UserId,
    },
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
