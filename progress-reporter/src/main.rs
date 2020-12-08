use parity_wordlist::random_phrase;
use std::{thread, time};

use interactive_exeunit::messages::{Info, Messages, Progress};
use yarapi::rest::streaming::send_to_guest;

fn main() {
    for i in 1..17 {
        let sleep_time = time::Duration::from_secs(2);
        thread::sleep(sleep_time);

        let progress: f64 = (i as f64) / 16.0;
        send_to_guest(&Messages::Progress(Progress { value: progress })).ok();

        let funny_text = random_phrase(3);
        send_to_guest(&Messages::Info(Info {
            message: funny_text,
        }))
        .ok();

        println!("\n");
    }
}
