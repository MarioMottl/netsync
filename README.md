# NetSync

NetSync is a remote control framework written in Rust that enables you to synchronize files across multiple testbench machines using a master–slave architecture. The master monitors a specified directory for changes and pushes update commands to connected slave clients, which then execute the appropriate actions.

## Features

- **Master–Slave Architecture:** One master sends commands to many slave clients.
- **Real-Time File Monitoring:** Uses the [notify](https://crates.io/crate/notify) crate to watch a directory for file changes.
- **TCP-Based Communication:** The master broadcasts commands via TCP sockets.
- **Shared Command Protocol:** A shared module (`src/commands/`) defines a universal set of commands (e.g., Update, Ping, Custom) along with serialization/deserialization logic.
- **Customizable Command Execution:** Commands include methods to execute on both server and client sides, and the client execution can utilize context (e.g., IP and port) for advanced operations.
- **Error Handling & Logging:** Built with [anyhow](https://crates.io/crate/anyhow) for error handling, [clap](https://crates.io/crate/clap) for argument parsing, and [env_logger](https://crates.io/crate/env_logger) with [log](https://crates.io/crate/log) for logging.

## Project Structure

```
netsync/
├── Cargo.toml
└── src
    ├── master.rs      # Master binary code
    ├── slave.rs       # Slave binary code
    └── commands
        ├── mod.rs     # Module root for shared commands
        └── commands.rs# Shared command definitions and protocol
```

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (with Cargo) installed.

### Clone and Build

Clone the repository:

```bash
git clone https://github.com/MarioMottl/netsync.git
cd netsync
```

Build the master binary:

```bash
cargo build --bin master --release
```

Build the slave binary:

```bash
cargo build --bin slave --release
```

## Usage

### Master

The master monitors a repository folder and sends commands to connected slaves when changes occur.

Run the master binary using command-line arguments or environment variables:

```bash
# Using command-line arguments:
./target/release/master --repo-path "C:\path\to\watch" --port 9000

# Or set environment variables and run without arguments:
export WATCH_PATH="C:\path\to\watch"
export WATCH_PORT=9000
./target/release/master
```

### Slave

The slave connects to the master and listens for incoming commands.

Run the slave binary with the required parameters:

```bash
# Using command-line arguments:
./target/release/slave --master-addr "192.168.1.100:9000" --client-ip "127.0.0.1" --client-port 8000

# Or set environment variables:
export MASTER_ADDR="192.168.1.100:9000"
export CLIENT_IP="127.0.0.1"
export CLIENT_PORT=8000
./target/release/slave
```

## Extending the Protocol

The shared command protocol is defined in `src/commands/commands.rs`. To add or modify commands:

- Update the `Command` enum variants.
- Implement new behaviors in `execute_on_server` and/or `execute_on_client`.
- Rebuild the project so both master and slave use the updated protocol.

## Contributing

Contributions are welcome! Please open issues or submit pull requests on the [GitHub repository](https://github.com/yourusername/netsync).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
