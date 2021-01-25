use chrono::{DateTime, Utc};
use futures::future::ready;
use futures::StreamExt;
use rustyline::Editor;
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
use yarapi::rest::streaming::{MessagingRequestor, ResultStream, StreamingActivity};
use yarapi::rest::{self, Activity};
use yarapi::ya_agreement_utils::{constraints, ConstraintKey, Constraints};

use prophecy_on_demand::Messages;

const PACKAGE: &str =
    "hash:sha3:8f09d137a837c376cd7c2b9c47e5390b1774c8895ac83bb5f9144d88:http://yacn.dev.golem.network:8000/progress-reporter-0.2.12";

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
#[structopt(rename_all = "kebab-case")]
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
            interact(activity.clone()).await?;
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

pub async fn interact(activity: Arc<DefaultActivity>) -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::unbounded_channel::<Messages>();

    let batch = activity
        .run_streaming(
            "/bin/prophecy-on-demand",
            vec!["--messages-dir".to_string(), "/messages".to_string()],
        )
        .await?
        .debug(".debug")?;

    tokio::spawn(events_tracker(receiver));
    tokio::task::spawn_local(cmdline_commands(activity.clone()));

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
    while let Some(msg) = receiver.recv().await {
        match msg {
            Messages::ProphecyResult { message } => println!("{}", message),
            _ => (),
        }
    }
}

async fn cmdline_commands(activity: Arc<DefaultActivity>) {
    let messaging = MessagingRequestor::new(activity, &PathBuf::from("/messages"));
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline("$ ");
        let input = match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str());
                line
            }
            Err(_) => break,
        };

        match &input[..] {
            "get" => messaging
                .send(&Messages::GetProphecy)
                .await
                .map_err(|e| log::warn!("Sending [GetProphecy]: {}", e))
                .ok(),
            "exit" => {
                messaging
                    .send(&Messages::Finish)
                    .await
                    .map_err(|e| log::warn!("Sending [Finish]: {}", e))
                    .ok();
                break;
            }
            _ => None,
        };
    }
}
