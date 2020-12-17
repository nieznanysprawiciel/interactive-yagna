use derive_more::From;
use serde::{Deserialize, Serialize};

use yarapi::rest::streaming::ExeUnitMessage;

#[derive(Serialize, Deserialize)]
pub struct GetProphecy {}

#[derive(Serialize, Deserialize, From)]
pub enum Messages {
    GetProphecy,
    Finish,
    ProphecyResult { message: String },
}

impl ExeUnitMessage for Messages {}
