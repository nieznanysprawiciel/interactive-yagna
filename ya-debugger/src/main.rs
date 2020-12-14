use chrono::{Duration, Utc};
use humantime;
use structopt::StructOpt;

use ya_client::web::WebClient;

use yarapi::rest::{self, Activity, Session};

#[derive(StructOpt)]
struct ActivityParams {
    #[structopt(long, env)]
    agreement_id: String,
    #[structopt(long, env)]
    activity_id: String,
}

#[derive(StructOpt)]
enum Commands {
    ListAgreements {
        #[structopt(long, parse(try_from_str = humantime::parse_duration), default_value = "24h")]
        since: std::time::Duration,
    },
}

#[derive(StructOpt)]
struct Args {
    #[structopt(long, env, default_value = "community.3")]
    subnet: String,
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
        Commands::ListAgreements { since } => {
            list_agreements(session, chrono::Duration::from_std(since)?).await?
        }
    };
    Ok(())
}

async fn list_agreements(session: Session, since: Duration) -> anyhow::Result<()> {
    let market = session.market()?;

    let timestamp = Utc::now() - since;
    let agreements = market.list_agreements(&timestamp, None).await?;

    println!("Agreements since {:#?}", timestamp);
    println!("{:#?}", agreements);
    Ok(())
}
