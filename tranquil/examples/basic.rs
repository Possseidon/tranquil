use std::sync::atomic::AtomicUsize;

use tranquil::Tranquil;

#[derive(Default)]
struct Bot {
    ping_count: AtomicUsize,
}

struct Ping;

// TODO: derive
// impl Command for Ping {
//     const NAME: &str = "ping";

//     type Autocomplete = NoAutocomplete;

//     fn create() -> CreateCommand {
//         CreateCommand::new(Self::NAME)
//             .description("Pings the bot and shows the number of times it got pinged")
//             .add_option(CreateCommandOption::new(
//                 CommandOptionType::User,
//                 "user",
//                 "tolle description",
//             ))
//     }

//     fn resolve(_interaction: &mut CommandInteraction) -> Result<Self> {
//         Ok(Self)
//     }

//     fn resolve_autocomplete(_data: &mut CommandInteraction) -> Result<Self::Autocomplete> {
//         Err(DiscordError::NotAutocompletable)?
//     }
// }

// impl Run<Ping> for Bot {
//     async fn run(
//         &self,
//         _command: Ping,
//         ctx: &Context,
//         interaction: CommandInteraction,
//     ) -> Result<()> {
//         let n = self.ping_count.fetch_add(1, atomic::Ordering::Relaxed) + 1;
//         interaction
//             .create_response(
//                 ctx,
//                 CreateInteractionResponse::Message(
//                     CreateInteractionResponseMessage::new().content(format!("Pong #{n}")),
//                 ),
//             )
//             .await?;
//         Ok(())
//     }
// }

struct Register {
    name: String,
}

// TODO: derive
// impl Modal for Register {
//     fn resolve(interaction: &mut ModalInteraction) -> Self {
//         let ActionRowComponent::InputText(input_text) =
//             interaction.data.components[0].components[0]
//         else {
//             panic!()
//         };
//         Self {
//             name: input_text.value.unwrap(),
//         }
//     }
// }

// impl Submit<Register> for Bot {
//     fn submit(
//         &self,
//         modal: Register,
//         ctx: Context,
//         interaction: serenity::all::ModalInteraction,
//     ) -> impl Future<Output = std::result::Result<()>> + Send {
//         todo!()
//     }
// }

#[tokio::main]
async fn main() {
    // Load a .env file if one exists
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        panic!("{error}")
    }

    // Log to console based on the RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    // Setup tranquil
    let tranquil = Tranquil::new(Bot::default())
    // .command::<Ping>()
    ;

    // Setup serenity using tranquil as a framework
    // let mut client = Client::builder(
    //     std::env::var("DISCORD_TOKEN").unwrap(),
    //     GatewayIntents::empty(),
    // )
    // .framework(tranquil)
    // .await
    // .unwrap();

    // Run the Bot until it fully crashes or Ctrl+C is hit
    // select! {
    //     result = client.start() => result.unwrap(),
    //     result = signal::ctrl_c() => result.unwrap(),
    // }

    // Gracefully shutdown
    // client.shard_manager.shutdown_all().await;
}
