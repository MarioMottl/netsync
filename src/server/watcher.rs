use crate::commands::{Command, serialize_command};
use crate::server::Clients;
use anyhow::Result;
use log::{error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn run_watcher(clients: Clients, repo_path: &str) -> Result<()> {
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
