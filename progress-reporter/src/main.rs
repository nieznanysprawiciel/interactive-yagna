use parity_wordlist::random_phrase;
use std::{thread, time};
use structopt::StructOpt;

use interactive_exeunit::messages::{Info, Messages, Progress};
use yarapi::rest::streaming::send_to_guest;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Args {
    #[structopt(long, env)]
    num_events: u32,
}

fn main() {
    let args = Args::from_args();

    for i in 1..=args.num_events {
        let sleep_time = time::Duration::from_secs(2);
        thread::sleep(sleep_time);

        let progress: f64 = (i as f64) / args.num_events as f64;
        send_to_guest(&Messages::Progress(Progress { value: progress })).ok();

        let funny_text = random_phrase(3);
        send_to_guest(&Messages::Info(Info {
            message: funny_text,
        }))
        .ok();

        println!("\n");
    }
}
