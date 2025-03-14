#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};

#[derive(Debug)]
pub struct ClientContext {
    pub ip: String,
    pub port: u16,
    // Add more fields if needed.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Update,
    Ping,
    Custom { data: String },
}

impl Command {
    pub fn execute_on_server(&self) -> Result<()> {
        match self {
            Command::Update => {
                println!("Server executing update command.");
                Ok(())
            }
            Command::Ping => {
                println!("Server received ping command.");
                Ok(())
            }
            Command::Custom { data } => {
                println!("Server executing custom command with data: {}", data);
                Ok(())
            }
        }
    }

    pub fn execute_on_client(&self, ctx: &ClientContext) -> Result<()> {
        match self {
            Command::Update => {
                println!(
                    "Client executing update command. Using context IP: {} and Port: {}",
                    ctx.ip, ctx.port
                );
                Ok(())
            }
            Command::Ping => {
                println!("Client received ping command.");
                Ok(())
            }
            Command::Custom { data } => {
                println!("Client executing custom command with data: {}", data);
                Ok(())
            }
        }
    }
}

pub fn serialize_command(cmd: &Command) -> Result<Vec<u8>> {
    let bytes = to_vec(cmd)?;
    Ok(bytes)
}

pub fn deserialize_command(data: &[u8]) -> Result<Command> {
    let cmd = from_slice(data)?;
    Ok(cmd)
}
