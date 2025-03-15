# NetSync

NetSync is a remote control framework written in Rust that enables you to synchronize files across multiple testbench machines using a master–slave architecture. The master monitors a specified directory for changes and pushes update commands to connected slave clients, which then execute the appropriate actions.

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
./target/release/slave --master-addr "192.168.1.100:9000"

# Or set environment variables:
export MASTER_ADDR="192.168.1.100:9000"
export CLIENT_IP="127.0.0.1"
export CLIENT_PORT=8000
./target/release/slave
```
