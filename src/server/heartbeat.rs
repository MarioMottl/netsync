use crate::commands::{Command, serialize_command};
use crate::server::Clients;
use log::{error, info};
use std::io::Write;
use std::thread;
use std::time::Duration;

pub fn start_heartbeat(clients: Clients) {
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
            for peer in to_remove {
                info!("Removing client {}", peer);
                clients_lock.remove(&peer);
            }
        }
    }
}
