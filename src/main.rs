use clap::{Args, Parser, Subcommand};
use eyre::Result;
use futures_util::sink::SinkExt;
use http::Uri;
use nostr::message::RelayMessage;
use std::str::FromStr;
use tokio_websockets::{ClientBuilder, Message};

use crate::nostr::message::parse_relay_message;

mod nostr;

// Goal: read a user's posts; post simple messages
// Next up:
// - [ ] user-friendly event printing
// - cli subcommands:
//   - [X] keygen
//   - [ ] fetch --pubkey --limit --id
//   - [ ] post --priv-key --content --url
// - [ ] find all messages by users i'm following

// const _REQ_FIREHOSE: &str = r###"["REQ","mainFeed 1848",{"since":1672260784,"kinds":[1,2],"limit":5000}]"###;
const REQ_FRIEND: &str = r###"["REQ","adhoc 8004",{"authors":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"],"kinds":[0]},{"authors":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"],"kinds":[1]},{"authors":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e","29f63b70d8961835b14062b195fc7d84fa810560b36dde0749e4bc084f0f8952"],"kinds":[3]},{"#p":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"],"kinds":[3]}]"###;

// https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
// Looks like programmatic cli arg generation might be the way to go.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // #[arg(short, long, default_value_t = ("wss://relay.nostr.info/").to_string() )]
    // url: String,
    #[command(subcommand)]
   cmd: Cmd,
}

#[derive(Args, Debug)]
struct PullArgs {
    #[arg(short, long, default_value_t = ("wss://relay.nostr.info/").to_string() )]
    url: String,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    #[command(about = "Pull latest messages")]
    Pull(PullArgs),
    #[command(about = "Generate new key pair")]
    Gen,
}

fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn parse_response(raw: &str) -> Result<RelayMessage> {
    let msg = parse_relay_message(&raw)?;

    match msg {
        RelayMessage::Event(ref msg) => println!("{}", msg.event),
        RelayMessage::Notice(_) => println!("Notice: {:?}", msg),
        RelayMessage::Unknown(_) => println!("Unknown: {:?}", msg),
    }

    Ok(msg)
}

async fn firehose(uri: Uri) -> Result<(), tokio_websockets::Error> {
    println!("Connecting to url: {:?}", &uri);
    let uri_clone = uri.clone();
    let mut client = ClientBuilder::from_uri(uri).connect().await?;

    client.send(Message::text(String::from(REQ_FRIEND))).await?;
    println!("REQ_FRIEND sent to relay: {:?}", &uri_clone);

    while let Some(Ok(msg)) = client.next().await {
        if let Ok(text) = msg.as_text() {
            println!("msg: {}", text);
            let _message = parse_response(text);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    println!("Running with args: {:?}", args);
    match args.cmd {
        Cmd::Pull(pull_args) => {
            let uri = Uri::from_str(&pull_args.url)?;
            let result = firehose(uri).await;
            match result {
                Ok(_) => println!("all good here!"),
                Err(e) => panic!("Shit hit the fan: {:?}", e)
            }
        }
        Cmd::Gen => {
            println!("Generating Nostr key pair...");
            let (secret_key, public_key) = nostr::event::gen_keypair().expect("Failed to generate keypair");
            println!("secret: {}", secret_key.display_secret());
            println!("public: {}", public_key);
        }
    }

    Ok(())
}
