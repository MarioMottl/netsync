use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Update,
    Ping,
    Identify { hostname: String },
    Custom { data: String },
}

pub fn serialize_command(cmd: &Command) -> Result<Vec<u8>> {
    let bytes = to_vec(cmd)?;
    Ok(bytes)
}

pub fn deserialize_command(data: &[u8]) -> Result<Command> {
    let cmd = from_slice(data)?;
    Ok(cmd)
}
