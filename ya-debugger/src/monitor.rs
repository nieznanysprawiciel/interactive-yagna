use futures::future::ready;
use futures::StreamExt;
use std::path::PathBuf;
use structopt::StructOpt;
use tabular::{Row, Table};

use ya_client_model::activity::RuntimeEventKind;
use yarapi::rest::activity::DefaultActivity;
use yarapi::rest::streaming::{ResultStream, StreamingBatch};
use yarapi::rest::Session;

use crate::attach::attach_debugger;
use crate::display::EnableDisplay;

#[derive(StructOpt)]
pub struct Activity {
    #[structopt(env, long = "id")]
    pub activity_id: String,
    #[structopt(subcommand)]
    pub command: ActivityCommands,
}

#[derive(StructOpt)]
pub enum ActivityCommands {
    Monitor,
    Attach,
    CaptureOutput {
        #[structopt(env, long)]
        batch_id: String,
    },
}

pub async fn run_activity_command(session: Session, params: Activity) -> anyhow::Result<()> {
    let activity = session.attach_to_activity(&params.activity_id).await?;
    match params.command {
        ActivityCommands::Monitor => {
            activity_status(activity).await?;
        }
        ActivityCommands::Attach => {
            attach_debugger(session, activity).await?;
        }
        ActivityCommands::CaptureOutput { batch_id } => {
            capture_output(session, activity, batch_id).await?;
        }
    };
    Ok(())
}

pub async fn activity_status(activity: DefaultActivity) -> anyhow::Result<()> {
    let state = activity.get_state().await?;

    let mut table = Table::new("{:>}  {:<} {:<} {:<}");
    table.add_row(
        Row::new()
            .with_cell("State")
            .with_cell(state.state.display())
            .with_cell(state.reason.display())
            .with_cell(state.error_message.display()),
    );

    // Needs fix in ya-client
    // let usage = activity.get_usage().await?;
    // table.add_row(
    //     Row::new()
    //         .with_cell("Usage")
    //         .with_cell(usage.current_usage.unwrap_or(vec![]).display())
    //         .with_cell("")
    //         .with_cell(""),
    // );

    if state.alive() {
        let command = activity.get_running_command().await?;

        table.add_row(
            Row::new()
                .with_cell("Command")
                .with_cell(command.command)
                .with_cell(command.params.unwrap_or(vec![]).display())
                .with_cell(command.progress.display()),
        );
    }

    print!("{}", table);
    Ok(())
}

pub async fn capture_output(
    _session: Session,
    activity: DefaultActivity,
    batch_id: String,
) -> anyhow::Result<()> {
    let batch = StreamingBatch::from(activity.attach_to_batch(&batch_id));
    batch
        .stream()
        .await?
        .forward_to_std()
        .forward_to_file(
            &PathBuf::from(".debug-stdout.txt"),
            &PathBuf::from(".debug-stderr.txt"),
        )?
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
    Ok(())
}
