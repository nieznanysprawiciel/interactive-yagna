use chrono::{Duration, Utc};
use humantime;
use structopt::StructOpt;

use yarapi::rest::Session;

#[derive(StructOpt)]
pub struct Agreement {
    #[structopt(subcommand)]
    pub command: AgreementCommands,
}

#[derive(StructOpt)]
pub enum AgreementCommands {
    List {
        #[structopt(long, parse(try_from_str = humantime::parse_duration), default_value = "24h")]
        since: std::time::Duration,
    },
    ListActive {
        #[structopt(long, parse(try_from_str = humantime::parse_duration), default_value = "100days")]
        since: std::time::Duration,
    },
}

pub async fn run_agreement_command(session: Session, cmd: AgreementCommands) -> anyhow::Result<()> {
    match cmd {
        AgreementCommands::List { since } => {
            list_agreements(session, chrono::Duration::from_std(since)?).await?
        }
        AgreementCommands::ListActive { since } => {
            list_active_agreements(session, chrono::Duration::from_std(since)?).await?
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

async fn list_active_agreements(session: Session, since: Duration) -> anyhow::Result<()> {
    let market = session.market()?;

    let timestamp = Utc::now() - since;
    let agreements = market.list_active_agreements(&timestamp, None).await?;

    println!("Active Agreements since {:#?}", timestamp);
    println!("{:#?}", agreements);
    Ok(())
}
