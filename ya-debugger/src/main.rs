mod agreement;
mod display;
mod monitor;

use structopt::StructOpt;

use ya_client::web::WebClient;
use yarapi::rest::{self};

use crate::agreement::{run_agreement_command, Agreement};
use crate::monitor::{run_activity_command, Activity};

#[derive(StructOpt)]
enum Commands {
    Agreement(Agreement),
    Activity(Activity),
}

#[derive(StructOpt)]
struct Args {
    #[structopt(long, env = "YAGNA_APPKEY")]
    appkey: String,
    #[structopt(subcommand)]
    command: Commands,
}

#[actix_rt::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = Args::from_args();
    std::env::set_var("RUST_LOG", "info");
    env_logger::builder()
        .filter_module("yarapi::drop", log::LevelFilter::Off)
        .filter_module("ya_service_bus::connection", log::LevelFilter::Off)
        .filter_module("ya_service_bus::remote_router", log::LevelFilter::Off)
        .init();

    let client = WebClient::with_token(&args.appkey);
    let session = rest::Session::with_client(client.clone());

    match args.command {
        Commands::Agreement(agreement_cmd) => {
            run_agreement_command(session, agreement_cmd.command).await?
        }
        Commands::Activity(activity_cmd) => run_activity_command(session, activity_cmd).await?,
    };

    Ok(())
}
