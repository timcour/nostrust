use std::str::FromStr;

use futures_util::sink::SinkExt;
use clap::Parser;
use http::Uri;
use tokio_websockets::{ClientBuilder, Error, Message};

mod nostr;

const REQ_FIREHOSE: &str = r###"["REQ","mainFeed 1848",{"since":1672260784,"kinds":[1,2],"limit":5000}]"###;

// https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = ("wss://relay.nostr.info/").to_string() )]
    url: String
}

async fn firehose(uri: Uri) -> Result<(), Error> {
    println!("Connecting to url: {:?}", uri);
    let mut client = ClientBuilder::from_uri(uri).connect().await?;

    client.send(Message::text(String::from(REQ_FIREHOSE))).await?;
    println!("Message sent");

    while let Some(Ok(msg)) = client.next().await {
        if let Ok(text) = msg.as_text() {
            println!("{:?}", text);
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

    let event = nostr::event::Event {id: "someid".to_string()};
    println!("my event, {:?}", event);
}
