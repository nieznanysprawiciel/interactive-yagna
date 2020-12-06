use chrono::{Utc, DateTime};
use std::ops::Add;
use std::sync::Arc;
use structopt::StructOpt;

use ya_client::web::WebClient;
use yarapi::requestor::Image;
use yarapi::rest::{self, Activity};
use yarapi::ya_agreement_utils::{constraints, ConstraintKey, Constraints};
use ya_client_model::market::NewDemand;

/// TODO: Replace with real image.
const PACKAGE: &str =
    "hash:sha3:b491514aa88dc7f79ed461358cf9ea9c63775da591312f2f1a1dc43d:http://yacn.dev.golem.network:8000/ya-zksync-prover-0.2.3";

pub fn create_demand(deadline: DateTime<Utc>, subnet: &str) -> NewDemand {
    log::info!("Using subnet: {}", subnet);

    let ts = deadline.timestamp_millis();
    let properties = serde_json::json!({
        "golem.node.id.name": "interactive-example",
        "golem.node.debug.subnet": subnet,
        "golem.srv.comp.task_package": PACKAGE,
        "golem.srv.comp.expiration": ts
    });

    let constraints = constraints![
        "golem.runtime.name" == Image::GVMKit((0, 2, 3).into()).runtime_name(),
        "golem.node.debug.subnet" == subnet
    ].to_string();

    NewDemand {
        properties,
        constraints
    }
}

#[derive(StructOpt)]
struct Args {
    #[structopt(long, env, default_value = "devnet-alpha.3")]
    subnet: String,
    #[structopt(long, env = "YAGNA_APPKEY")]
    appkey: String,
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
    let market = session.market()?;

    let deadline = Utc::now().add(chrono::Duration::minutes(25));
    let demand = create_demand(deadline, &args.subnet);

    let subscription = market.subscribe_demand(demand.clone()).await?;
    log::info!("Created subscription [{}]", subscription.id().as_ref());

    let agreements = subscription.negotiate_agreements(demand, 1, deadline).await?;
    let activity = Arc::new(session.create_activity(&agreements[0]).await?);

    session
        .with(async {
            log::info!("Deploying image and starting ExeUnit...");
            if let Err(e) = activity.execute_commands(
                vec![
                    rest::ExeScriptCommand::Deploy {},
                    rest::ExeScriptCommand::Start { args: vec![] },
                ],
            )
                .await
            {
                log::error!("Failed to initialize yagna task. Error: {}.", e);
                return Ok(());
            };

            log::info!("Image deployed. ExeUnit started.");
            Ok(())
        })
        .await
        .unwrap_or_else(|| anyhow::bail!("ctrl-c caught"))
        .map_err(|e| log::info!("{}", e))
        .ok();

    log::info!("Destroying activity..");
    activity
        .destroy()
        .await
        .map_err(|e| log::error!("Can't destroy activity. Error: {}", e))
        .ok();

    Ok(())
}
