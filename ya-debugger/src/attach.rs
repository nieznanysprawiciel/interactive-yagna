use yarapi::rest::activity::DefaultActivity;
use yarapi::rest::Session;

pub async fn attach_debugger(session: Session, activity: DefaultActivity) -> anyhow::Result<()> {
    drop(session);
    drop(activity);
    unimplemented!()
}
