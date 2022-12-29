use clap::Parser;

mod nostr;

// https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = ("wss://relay.damus.io").to_string() )]
    url: String
}

fn main() {
    let args = Cli::parse();
    println!("Running with args: {:?}", args);

    let event = nostr::event::Event {id: "someid".to_string()};
    println!("my event, {:?}", event);
}
