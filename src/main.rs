use std::str::FromStr;

use eyre::Result;
use futures_util::sink::SinkExt;
use clap::Parser;
use http::Uri;
use nostr::message::RelayMessage;
use tokio_websockets::{ClientBuilder, Message};

use crate::nostr::message::parse_relay_message;

mod nostr;

const _REQ_FIREHOSE: &str = r###"["REQ","mainFeed 1848",{"since":1672260784,"kinds":[1,2],"limit":5000}]"###;
const REQ_FRIEND: &str = r###"["REQ","adhoc 8004",{"authors":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"],"kinds":[0]},{"authors":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"],"kinds":[1]},{"authors":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e","29f63b70d8961835b14062b195fc7d84fa810560b36dde0749e4bc084f0f8952"],"kinds":[3]},{"#p":["b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"],"kinds":[3]}]"###;

// https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = ("wss://relay.nostr.info/").to_string() )]
    url: String
}

fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn parse_response(raw: &str) -> Result<RelayMessage> {
    let msg = parse_relay_message(&raw)?;
    println!("Parsed message: {:?}", msg);
    Ok(msg)
}

async fn firehose(uri: Uri) -> Result<(), tokio_websockets::Error> {
    println!("Connecting to url: {:?}", uri);
    let mut client = ClientBuilder::from_uri(uri).connect().await?;

    client.send(Message::text(String::from(REQ_FRIEND))).await?;
    println!("Message sent");

    while let Some(Ok(msg)) = client.next().await {
        if let Ok(text) = msg.as_text() {
            let message = parse_response(text);
            println!("{:?}", message);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    println!("Running with args: {:?}", args);

    let uri = Uri::from_str(&args.url).expect("Bad URI");
    let result = firehose(uri).await;
    match result {
        Ok(_) => println!("all good here!"),
        Err(e) => panic!("Shit hit the fan: {:?}", e)
    }
}
