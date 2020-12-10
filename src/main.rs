use chrono::{DateTime, Utc};
use futures::future::ready;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::mpsc;

use ya_client::web::WebClient;
use ya_client_model::activity::RuntimeEventKind;
use ya_client_model::market::NewDemand;

use yarapi::requestor::Image;
use yarapi::rest::activity::DefaultActivity;
use yarapi::rest::streaming::{ResultStream, StreamingActivity};
use yarapi::rest::{self, Activity};
use yarapi::ya_agreement_utils::{constraints, ConstraintKey, Constraints};

use interactive_exeunit::messages::{Info, Messages, Progress};

const PACKAGE: &str =
    "hash:sha3:1b93678011c94a883605634eb4636a27a2ac6459816f48647b6ab968:http://yacn.dev.golem.network:8000/progress-reporter-0.1.1";

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
    ]
    .to_string();

    NewDemand {
        properties,
        constraints,
    }
}

#[derive(StructOpt)]
struct Args {
    #[structopt(long, env, default_value = "community.3")]
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
        .filter_module("ya_service_bus::remote_router", log::LevelFilter::Off)
        .init();

    let client = WebClient::with_token(&args.appkey);
    let session = rest::Session::with_client(client.clone());
    let market = session.market()?;

    let deadline = Utc::now().add(chrono::Duration::minutes(25));
    let demand = create_demand(deadline, &args.subnet);

    let subscription = market.subscribe_demand(demand.clone()).await?;
    log::info!("Created subscription [{}]", subscription.id().as_ref());

    let agreements = subscription
        .negotiate_agreements(demand, 1, deadline)
        .await?;
    let activity = Arc::new(session.create_activity(&agreements[0]).await?);

    session
        .with(async {
            log::info!("Deploying image and starting ExeUnit...");
            if let Err(e) = activity
                .execute_commands(vec![
                    rest::ExeScriptCommand::Deploy {},
                    rest::ExeScriptCommand::Start { args: vec![] },
                ])
                .await
            {
                log::error!("Failed to initialize yagna task. Error: {}.", e);
                return Ok(());
            };

            log::info!("Image deployed. ExeUnit started.");
            monitor_progress(activity.clone()).await?;
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

pub async fn monitor_progress(activity: Arc<DefaultActivity>) -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::unbounded_channel::<Messages>();

    tokio::spawn(events_tracker(receiver));

    let batch = activity
        .run_streaming("/bin/progress-reporter", vec![])
        .await?;
    batch
        .stream()
        .await?
        .forward_to_file(&PathBuf::from("stdout.txt"), &PathBuf::from("stderr.txt"))?
        .capture_messages(sender)
        .take_while(|event| {
            ready(match &event.kind {
                RuntimeEventKind::Finished {
                    return_code,
                    message,
                } => {
                    let no_msg = "".to_string();
                    log::info!(
                        "ExeUnit finished with code {}, and message: {}",
                        return_code,
                        message.as_ref().unwrap_or(&no_msg)
                    );
                    false
                }
                _ => true,
            })
        })
        .for_each(|_| ready(()))
        .await;

    batch.wait_for_finish().await?;
    Ok(())
}

async fn events_tracker(mut receiver: mpsc::UnboundedReceiver<Messages>) {
    let bar_max: u64 = 100;
    let bar = ProgressBar::new(bar_max);

    bar.set_style(
        ProgressStyle::default_bar()
            .template("{prefix}: {bar:40.cyan/blue} {pos:>7}/{len:7} \nThis will happen to you tomorrow: {msg}")
            .progress_chars("##-"),
    );
    bar.set_prefix("Collecting prophecies");
    bar.set_position(0);

    while let Some(msg) = receiver.recv().await {
        match msg {
            Messages::Progress(Progress { value }) => {
                bar.set_position((bar_max as f64 * value) as u64)
            }
            Messages::Info(Info { message }) => bar.set_message(&message),
        }
    }

    bar.set_position(bar_max);
    //bar.finish();
}
