use crate::commands::{Command, serialize_command};
use crate::server::ClientInfo;
use anyhow::Result;
use log::{error, info};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub fn start_repl(clients: Arc<Mutex<HashMap<String, ClientInfo>>>) -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?; // Ensure prompt is displayed

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            // EOF or read error
            break;
        }
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        match parse_command(line) {
            ReplCmd::List => list_clients(&clients),
            ReplCmd::Send { hostname, data } => {
                send_custom_command(&clients, &hostname, &data);
            }
            ReplCmd::Quit => {
                info!("Quitting REPL...");
                break;
            }
            ReplCmd::Unknown => {
                println!("Unknown command. Try 'list', 'send <hostname> <data>', or 'quit'.");
            }
        }
    }
    Ok(())
}

enum ReplCmd {
    List,
    Send { hostname: String, data: String },
    Quit,
    Unknown,
}

fn parse_command(line: &str) -> ReplCmd {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return ReplCmd::Unknown;
    }
    match tokens[0] {
        "list" => ReplCmd::List,
        "send" if tokens.len() >= 3 => {
            let hostname = tokens[1].to_string();
            let data = tokens[2..].join(" ");
            ReplCmd::Send { hostname, data }
        }
        "quit" => ReplCmd::Quit,
        _ => ReplCmd::Unknown,
    }
}

fn list_clients(clients: &Arc<Mutex<HashMap<String, ClientInfo>>>) {
    let lock = clients.lock().unwrap();
    println!("Currently connected clients:");
    for (peer, info) in lock.iter() {
        let host = info.hostname.as_deref().unwrap_or("<no-hostname>");
        println!(" - {} (hostname: {})", peer, host);
    }
}

fn send_custom_command(
    clients: &Arc<Mutex<HashMap<String, ClientInfo>>>,
    hostname: &str,
    data: &str,
) {
    let mut lock = clients.lock().unwrap();
    let maybe_peer = lock.iter_mut().find_map(|(peer, info)| {
        if let Some(ref h) = info.hostname {
            if h == hostname {
                return Some((peer.clone(), info));
            }
        }
        None
    });
    match maybe_peer {
        Some((peer, client_info)) => {
            let cmd = Command::Custom {
                data: data.to_string(),
            };
            match serialize_command(&cmd) {
                Ok(msg) => {
                    if let Err(e) = client_info.stream.write_all(&msg) {
                        error!("Failed to send custom command to {}: {}", peer, e);
                    } else {
                        info!("Sent custom command to {} (hostname: {})", peer, hostname);
                    }
                }
                Err(e) => {
                    error!("Failed to serialize custom command: {}", e);
                }
            }
        }
        None => {
            println!("No client found with hostname '{}'.", hostname);
        }
    }
}
