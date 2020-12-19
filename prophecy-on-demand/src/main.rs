use parity_wordlist::random_phrase;
use std::path::PathBuf;
use structopt::StructOpt;

mod messages;

use crate::messages::Messages;
use yarapi::rest::streaming::{send_to_guest, MessagingExeUnit};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Args {
    #[structopt(long, env)]
    messages_dir: String,
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    let messaging = MessagingExeUnit::new(&PathBuf::from(&args.messages_dir))?;
    let mut listener = messaging.listen::<Messages>();

    while let Some(message) = listener.recv().await {
        match message {
            Messages::Finish => return Ok(()),
            Messages::GetProphecy => send_prophecy(),
            _ => continue,
        }
    }
    Ok(())
}

fn send_prophecy() {
    let funny_text = random_phrase(3);
    println!("Debug print: {}\n", funny_text);

    send_to_guest(&Messages::ProphecyResult {
        message: funny_text,
    })
    .ok();
}
