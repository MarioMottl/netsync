use anyhow::Result;
use log::{error, info};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use crate::commands::{Command, deserialize_command, serialize_command};

pub fn run_slave(master_addr: &str, hostname: &str) -> Result<()> {
    loop {
        match TcpStream::connect(master_addr) {
            Ok(mut stream) => {
                info!("Connected to master at {}", master_addr);
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .expect("Failed to set timeout");

                send_identify(&mut stream, hostname);

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

fn send_identify(stream: &mut TcpStream, hostname: &str) {
    let cmd = Command::Identify {
        hostname: hostname.to_string(),
    };
    match serialize_command(&cmd) {
        Ok(msg) => {
            if let Err(e) = send_command(stream, &msg, "Identify") {
                error!("Error sending Identify command: {}", e);
            }
        }
        Err(e) => error!("Failed to serialize Identify command: {}", e),
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
    send_command(stream, &reply, "PONG")?;
    Ok(())
}

fn send_command(stream: &mut TcpStream, msg: &[u8], command_label: &str) -> Result<()> {
    if let Err(e) = stream.write_all(msg) {
        error!("Failed to send {} command: {}", command_label, e);
        Err(e.into())
    } else {
        info!("Sent {} command", command_label);
        Ok(())
    }
}
