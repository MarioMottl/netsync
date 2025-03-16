mod client;
mod commands;
mod logger;

use anyhow::Result;
use clap::Parser;
use hostname::get as get_hostname;
use log::info;
use logger::init_logger;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, env("MASTER_ADDR"), default_value = "192.168.1.100:9000")]
    master_addr: String,

    #[arg(long, env("CLIENT_HOSTNAME"), default_value = "")]
    client_hostname: String,
}

fn main() -> Result<()> {
    let _guard = init_logger(logger::LoggerMode::Console);
    let args = Args::parse();
    info!("Using master address: {}", args.master_addr);

    let hostname: String = if args.client_hostname.trim().is_empty() {
        get_hostname()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        args.client_hostname.clone()
    };
    info!("Client hostname: {}", hostname);

    client::run_slave(&args.master_addr, &hostname)
}
