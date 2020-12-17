use parity_wordlist::random_phrase;
use std::path::PathBuf;

use two_way_interaction::messages::Messages;
use yarapi::rest::streaming::{send_to_guest, MessagingExeUnit};

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let messaging = MessagingExeUnit::new(&PathBuf::from("/messages"))?;
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
