use serde::{Serialize, Deserialize};
use derive_more::From;

use yarapi::rest::streaming::ExeUnitMessage;

#[derive(Serialize, Deserialize)]
pub struct Progress {
    pub value: f64
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub message: String
}


#[derive(Serialize, Deserialize, From)]
pub enum Messages {
    Progress(Progress),
    Info(Info),
}

impl ExeUnitMessage for Messages {}

