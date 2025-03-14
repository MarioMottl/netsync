mod commands;

use crate::commands::{Command, serialize_command};
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, env("WATCH_PATH"), default_value = "C:\\path\\to\\watch")]
    repo_path: String,

    #[arg(short, long, env("WATCH_PORT"), default_value = "9000")]
    port: u16,
}

fn handle_client(stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    info!("Slave connected: {:?}", peer_addr);
    // In a real implementation you might handle heartbeat messages or actual command responses.
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn run_server(clients: Arc<Mutex<Vec<TcpStream>>>, port: u16) {
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind TCP listener");
    info!("Master server listening on port {}...", port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                {
                    let mut clients_lock = clients.lock().unwrap();
                    clients_lock.push(
                        stream
                            .try_clone()
                            .expect("Failed to clone slave connection"),
                    );
                }
                let stream_clone = stream
                    .try_clone()
                    .expect("Failed to clone connection for thread");
                thread::spawn(move || {
                    handle_client(stream_clone);
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn run_watcher(clients: Arc<Mutex<Vec<TcpStream>>>, repo_path: &str) -> Result<()> {
    let repo_path = Path::new(repo_path);
    if !repo_path.exists() {
        error!("Could not find path to watch: {:?}", repo_path);
        return Err(anyhow::anyhow!(
            "Could not find path to watch: {:?}",
            repo_path
        ));
    }

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) => {
                info!("File event: {:?}", event);
                // Create an update command.
                let cmd = Command::Update;
                let msg = match serialize_command(&cmd) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Serialization error: {}", e);
                        return;
                    }
                };

                let clients_lock = clients.lock().unwrap();
                for mut client in clients_lock.iter() {
                    if let Err(e) = client.write_all(&msg) {
                        error!(
                            "Failed to send update to {}: {}",
                            client.peer_addr().unwrap(),
                            e
                        );
                    }
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        },
        Config::default(),
    )?;

    watcher.watch(repo_path, RecursiveMode::Recursive)?;
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    info!("Using repo path: {}", args.repo_path);
    info!("Using port: {}", args.port);

    let clients = Arc::new(Mutex::new(Vec::new()));
    let clients_for_server = Arc::clone(&clients);
    let clients_for_watcher = Arc::clone(&clients);

    let port = args.port;
    thread::spawn(move || {
        run_server(clients_for_server, port);
    });

    run_watcher(clients_for_watcher, &args.repo_path)
}
