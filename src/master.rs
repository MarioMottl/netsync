mod commands;
mod heartbeat;
mod logger;
mod server;
mod watcher;

use crate::logger::init_logger;
use anyhow::Result;
use clap::Parser;
use heartbeat::start_heartbeat;
use log::info;
use server::{ClientInfo, run_server};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use watcher::run_watcher;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the repository to watch
    #[arg(short, long, env("WATCH_PATH"), default_value = "C:\\path\\to\\watch")]
    repo_path: String,

    /// Port to listen on
    #[arg(short, long, env("WATCH_PORT"), default_value = "9000")]
    port: u16,
}

fn main() -> Result<()> {
    init_logger("info");
    let args = Args::parse();
    info!("Using repo path: {}", args.repo_path);
    info!("Using port: {}", args.port);

    // Shared client map
    let clients: Arc<Mutex<HashMap<String, ClientInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    let clients_for_server = Arc::clone(&clients);
    let clients_for_heartbeat = Arc::clone(&clients);

    let port = args.port;

    // Start server thread that accepts connections.
    thread::spawn(move || {
        run_server(clients_for_server, port);
    });

    // Start heartbeat thread.
    thread::spawn(move || {
        start_heartbeat(clients_for_heartbeat);
    });

    // Run file watcher in main thread.
    run_watcher(clients, &args.repo_path)
}
