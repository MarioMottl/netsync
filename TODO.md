
# TODO: Implement Heartbeat Mechanism and Default Logging

## Problem Statement

- **Stale Connections on the Server:**  
  The master (server) currently does not remove connections when a slave disconnects unexpectedly. This leads to a buildup of stale connections over time.

- **Client Read-Timeout Issues:**  
  The client’s read timeout is triggered because the server never replies to its messages. This can be alleviated by implementing a heartbeat mechanism.

- **Default Logging Level:**  
  It is desirable to enable `RUST_LOG=info` by default so that users get useful runtime information without manually setting the environment variable.

## Proposed Solutions

### 1. Heartbeat Mechanism

**On the Server (Master):**

- **Periodic Heartbeat:**  
  Implement a periodic task (e.g., every 5 seconds) that sends a heartbeat message (such as a `PING` command) to all connected slave clients.
  
- **Tracking and Removal:**  
  - For each client, record the timestamp when the heartbeat was sent.  
  - When a client responds with a `PONG` (or an appropriate acknowledgment), update the last-seen timestamp.  
  - If a client fails to respond within a predefined timeout period (e.g., 10 seconds), mark that connection as stale and remove it from the active connections list.

- **Code Sketch (Server Side):**

  ```rust
  // Inside a separate thread dedicated to heartbeat in master.rs
  loop {
      {
          let mut clients_lock = clients.lock().unwrap();
          // Iterate over clients and send a heartbeat command
          for client in clients_lock.iter_mut() {
              let heartbeat_cmd = Command::Ping;
              if let Ok(msg) = serialize_command(&heartbeat_cmd) {
                  if let Err(e) = client.write_all(&msg) {
                      error!("Failed to send heartbeat to {}: {}", client.peer_addr().unwrap(), e);
                      // Optionally: Mark or remove the client here.
                  }
              }
              // Optionally: Update a last-seen timestamp for the client.
          }
      }
      thread::sleep(Duration::from_secs(5));
  }
  ```

**On the Client (Slave):**

- **Heartbeat Response:**  
  When the client receives a `PING` command from the master, it should immediately reply with a `PONG` command (or a dedicated heartbeat acknowledgment).
  
- **Handling Heartbeat in Read Loop:**  
  Modify the client's command processing logic to detect the `Ping` command and respond accordingly, so the read timeout is not triggered.

- **Code Sketch (Client Side):**

  ```rust
  // Inside the client's read loop in slave.rs:
  match deserialize_command(&buffer[..n]) {
      Ok(cmd) => {
          match cmd {
              Command::Ping => {
                  // Respond to the heartbeat immediately
                  let pong_cmd = Command::Custom { data: "PONG".to_string() }; // Alternatively, define a dedicated Pong variant
                  if let Ok(reply) = serialize_command(&pong_cmd) {
                      if let Err(e) = stream.write_all(&reply) {
                          error!("Failed to send pong: {}", e);
                      }
                  }
              },
              _ => {
                  // Process other commands normally
                  if let Err(e) = cmd.execute_on_client(&ctx) {
                      error!("Command execution error: {}", e);
                  }
              }
          }
      },
      Err(e) => error!("Failed to deserialize command: {}", e),
  }
  ```

### 2. Default Logging Level

**Goal:**  
Ensure that the logging level is set to `info` by default, so that developers and users see useful runtime logs without manual configuration.

**Approach:**  
Check for the presence of the `RUST_LOG` environment variable at the start of your main function. If it is not set, programmatically set it to `info`.

- **Code Example:**

  ```rust
  use std::env;

  fn init_logger() {
      if env::var("RUST_LOG").is_err() {
          env::set_var("RUST_LOG", "info");
      }
      env_logger::init();
  }

  fn main() {
      init_logger();
      // Continue with argument parsing and program logic...
  }
  ```

## Summary

- **Heartbeat Implementation:**  
  Add periodic heartbeat messages (`PING`) from the server and corresponding replies (`PONG`) from the client. This allows the server to identify and remove stale connections and prevents client read-timeouts.

- **Default Logging:**  
  Automatically set `RUST_LOG=info` if not already defined, ensuring that useful logs are output by default.

These changes will improve the reliability of your NetSync framework by maintaining active connections and providing consistent logging.
