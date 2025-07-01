mod commands;
mod logger;
mod server;

use anyhow::Result;
use clap::Parser;
use log::{error, info};
use server::{ClientInfo, run_server};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, env("WATCH_PATH"), default_value = "C:\\path\\to\\watch")]
    repo_path: String,

    #[arg(short, long, env("WATCH_PORT"), default_value = "9000")]
    port: u16,
}

fn main() -> Result<()> {
    let _guard = logger::init_logger(logger::LoggerMode::File);
    let args = Args::parse();
    info!("Using repo path: {}", args.repo_path);
    info!("Using port: {}", args.port);

    let clients: Arc<Mutex<HashMap<String, ClientInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    let clients_for_server = Arc::clone(&clients);
    let clients_for_heartbeat = Arc::clone(&clients);
    let clients_for_repl = Arc::clone(&clients);

    let port = args.port;

    thread::spawn(move || {
        run_server(clients_for_server, port);
    });

    thread::spawn(move || {
        server::start_heartbeat(clients_for_heartbeat);
    });

    thread::spawn({
        let repo_path = args.repo_path.clone();
        let c = Arc::clone(&clients);
        move || {
            if let Err(e) = server::run_watcher(c, &repo_path) {
                error!("Watcher error: {}", e);
            }
        }
    });

    server::start_repl(clients_for_repl).expect("Could not start REPL");
    Ok(())
}
