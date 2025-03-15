mod commands;
mod logger;

use crate::logger::init_logger;
use anyhow::Result;
use clap::Parser;
use commands::{ClientContext, deserialize_command};
use log::{error, info};
use std::io::Read;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, env("MASTER_ADDR"), default_value = "192.168.1.100:9000")]
    master_addr: String,

    #[arg(long, env("CLIENT_IP"), default_value = "127.0.0.1")]
    client_ip: String,

    #[arg(long, env("CLIENT_PORT"), default_value = "8000")]
    client_port: u16,
}

fn run_slave(master_addr: &str, ctx: &ClientContext) -> Result<()> {
    loop {
        match TcpStream::connect(master_addr) {
            Ok(mut stream) => {
                info!("Connected to master at {}", master_addr);
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .expect("Failed to set timeout");
                let mut buffer = [0; 1024];
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            info!("Master closed the connection.");
                            break;
                        }
                        Ok(n) => match deserialize_command(&buffer[..n]) {
                            Ok(cmd) => {
                                info!("Received command: {:?}", cmd);
                                if let Err(e) = cmd.execute_on_client(ctx) {
                                    error!("Command execution error: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize command: {}", e);
                            }
                        },
                        Err(e) => {
                            error!("Error reading from master: {}", e);
                            break;
                        }
                    }
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

fn main() -> Result<()> {
    init_logger("info");
    let args = Args::parse();
    info!("Using master address: {}", args.master_addr);

    let ctx = ClientContext {
        ip: args.client_ip,
        port: args.client_port,
    };

    run_slave(&args.master_addr, &ctx)?;
    Ok(())
}
