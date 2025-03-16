use crate::commands::{Command, deserialize_command};
use log::{error, info};
use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Information about a connected client.
pub struct ClientInfo {
    pub stream: TcpStream,
    pub last_heartbeat: Instant,
    pub hostname: Option<String>,
}

pub type Clients = Arc<Mutex<HashMap<String, ClientInfo>>>;

pub fn run_server(clients: Clients, port: u16) {
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
                    hostname: None,
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

pub fn handle_client(mut stream: TcpStream, peer: String, clients: Clients) {
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
                            if let Some(client) = clients.lock().unwrap().get_mut(&peer) {
                                client.last_heartbeat = Instant::now();

                                if let Some(ref hostname) = client.hostname {
                                    info!("Received PONG from hostname: {}", hostname);
                                } else {
                                    info!("Received PONG from peer: {}", peer);
                                }
                            }
                        }
                        Command::Identify { hostname } => {
                            info!("Received Identify from {}: hostname = {}", peer, hostname);
                            if let Some(client) = clients.lock().unwrap().get_mut(&peer) {
                                client.hostname = Some(hostname);
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
