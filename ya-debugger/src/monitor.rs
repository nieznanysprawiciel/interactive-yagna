use structopt::StructOpt;
use tabular::{Row, Table};

use yarapi::rest::activity::DefaultActivity;
use yarapi::rest::Session;

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
}

pub async fn run_activity_command(session: Session, params: Activity) -> anyhow::Result<()> {
    match params.command {
        ActivityCommands::Monitor => {
            let activity = session.attach_to_activity(&params.activity_id).await?;
            activity_status(activity).await?;
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
