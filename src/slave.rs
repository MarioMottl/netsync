mod commands;
mod logger;

use crate::logger::init_logger;
use anyhow::Result;
use clap::Parser;
use commands::{Command, deserialize_command, serialize_command};
use log::{error, info};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, env("MASTER_ADDR"), default_value = "192.168.1.100:9000")]
    master_addr: String,
}

fn run_slave(master_addr: &str) -> Result<()> {
    loop {
        match TcpStream::connect(master_addr) {
            Ok(mut stream) => {
                info!("Connected to master at {}", master_addr);
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .expect("Failed to set timeout");
                if let Err(e) = process_incoming_messages(&mut stream) {
                    error!("Connection error: {}", e);
                }
            }
            Err(e) => {
                error!(
                    "Failed to connect to master: {}. Retrying in 5 seconds...",
                    e
                );
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn process_incoming_messages(stream: &mut TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                info!("Master closed the connection.");
                break;
            }
            Ok(n) => {
                handle_message(&buffer[..n], stream)?;
            }
            Err(e) => {
                error!("Error reading from master: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn handle_message(data: &[u8], stream: &mut TcpStream) -> Result<()> {
    match deserialize_command(data) {
        Ok(cmd) => match cmd {
            Command::Ping => {
                info!("Received PING from master.");
                send_pong(stream)
            }
            Command::Update => {
                info!("Received UPDATE from master.");
                Ok(())
            }
            _ => {
                info!("Received command: {:?}", cmd);
                Ok(())
            }
        },
        Err(e) => {
            error!("Failed to deserialize command: {}", e);
            Ok(())
        }
    }
}

fn send_pong(stream: &mut TcpStream) -> Result<()> {
    let pong_cmd = Command::Custom {
        data: "PONG".to_string(),
    };
    let reply = serialize_command(&pong_cmd)?;
    stream.write_all(&reply)?;
    info!("Sent PONG reply.");
    Ok(())
}

fn main() -> Result<()> {
    init_logger("info");
    let args = Args::parse();
    info!("Using master address: {}", args.master_addr);
    run_slave(&args.master_addr)?;
    Ok(())
}
