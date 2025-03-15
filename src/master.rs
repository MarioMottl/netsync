mod commands;
mod logger;

use crate::logger::init_logger;
use anyhow::Result;
use clap::Parser;
use commands::{Command, deserialize_command, serialize_command};
use log::{error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, env("WATCH_PATH"), default_value = "C:\\path\\to\\watch")]
    repo_path: String,

    #[arg(short, long, env("WATCH_PORT"), default_value = "9000")]
    port: u16,
}

struct ClientInfo {
    stream: TcpStream,
    last_heartbeat: Instant,
}

fn run_server(clients: Arc<Mutex<HashMap<String, ClientInfo>>>, port: u16) {
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind TCP listener");
    info!("Master server listening on port {}...", port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|_| "unknown".into());
                info!("Slave connected: {}", peer);

                let client_info = ClientInfo {
                    stream: stream.try_clone().expect("Failed to clone stream"),
                    last_heartbeat: Instant::now(),
                };
                clients.lock().unwrap().insert(peer.clone(), client_info);

                let clients_clone = Arc::clone(&clients);
                let stream_clone = stream
                    .try_clone()
                    .expect("Failed to clone stream for thread");
                thread::spawn(move || {
                    handle_client(stream_clone, peer, clients_clone);
                });
            }
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }
}

fn handle_client(
    mut stream: TcpStream,
    peer: String,
    clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                info!("Connection closed by client: {}", peer);
                clients.lock().unwrap().remove(&peer);
                break;
            }
            Ok(n) => {
                if let Ok(cmd) = deserialize_command(&buffer[..n]) {
                    match cmd {
                        Command::Custom { data } if data == "PONG" => {
                            info!("Received PONG from {}", peer);
                            if let Some(client) = clients.lock().unwrap().get_mut(&peer) {
                                client.last_heartbeat = Instant::now();
                            }
                        }
                        _ => info!("Received unknown command from {}: {:?}", peer, cmd),
                    }
                }
            }
            Err(e) => {
                error!("Error reading from {}: {}", peer, e);
                clients.lock().unwrap().remove(&peer);
                break;
            }
        }
    }
}

fn start_heartbeat(clients: Arc<Mutex<HashMap<String, ClientInfo>>>) {
    loop {
        thread::sleep(Duration::from_secs(5));
        let heartbeat_cmd = Command::Ping;
        let msg = match serialize_command(&heartbeat_cmd) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to serialize heartbeat: {}", e);
                continue;
            }
        };

        let mut to_remove = Vec::new();
        {
            let mut clients_lock = clients.lock().unwrap();
            for (peer, client_info) in clients_lock.iter_mut() {
                if let Err(e) = client_info.stream.write_all(&msg) {
                    error!("Failed to send heartbeat to {}: {}", peer, e);
                    to_remove.push(peer.clone());
                }
                if client_info.last_heartbeat.elapsed() > Duration::from_secs(10) {
                    error!("Client {} has timed out", peer);
                    to_remove.push(peer.clone());
                }
            }
            // Remove stale clients
            for peer in to_remove {
                info!("Removing client {}", peer);
                clients_lock.remove(&peer);
            }
        }
    }
}

fn run_watcher(clients: Arc<Mutex<HashMap<String, ClientInfo>>>, repo_path: &str) -> Result<()> {
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
                let update_cmd = Command::Update;
                let msg = match serialize_command(&update_cmd) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to serialize update command: {}", e);
                        return;
                    }
                };
                let mut clients_lock = clients.lock().unwrap();
                for (peer, client_info) in clients_lock.iter_mut() {
                    if let Err(e) = client_info.stream.write_all(&msg) {
                        error!("Failed to send update to {}: {}", peer, e);
                    }
                }
            }
            Err(e) => error!("Watcher error: {:?}", e),
        },
        Config::default(),
    )?;
    watcher.watch(repo_path, RecursiveMode::Recursive)?;
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() -> Result<()> {
    init_logger("info");
    let args = Args::parse();
    info!("Using repo path: {}", args.repo_path);
    info!("Using port: {}", args.port);

    let clients: Arc<Mutex<HashMap<String, ClientInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    let clients_for_server = Arc::clone(&clients);
    let clients_for_heartbeat = Arc::clone(&clients);

    let port = args.port;
    thread::spawn(move || {
        run_server(clients_for_server, port);
    });

    thread::spawn(move || {
        start_heartbeat(clients_for_heartbeat);
    });

    run_watcher(clients, &args.repo_path)
}
