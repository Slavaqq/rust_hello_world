# Chat Server in Rust

This is a simple chat server application written in Rust. It allows multiple clients to connect and communicate with each other in real-time.

## Features

- Accept multiple client connections.
- Broadcast messages from one client to all other connected clients.
- [**NEW** use `parking_lot::Mutex`](https://crates.io/crates/parking_lot)
- [**NEW** use `env_logger` for logging](https://crates.io/crates/env_logger)

## Requirements

- Rust programming language installed. You can install Rust from [here](https://www.rust-lang.org/tools/install).

## Usage

### Arguments

- `hostname`: The hostname for the server to bind to. Default is `localhost`.
- `port`: The port for the server to listen on. Default is `11111`.

### Running the Server

1. Clone the repository:
    ```sh
    git clone <repository_url>
    ```
2. Navigate to the project directory:
    ```sh
    cd <project_directory>
    ```
3. Build the project:
    ```sh
    cargo build --release
    ```
4. Run the server:
    ```sh
    cargo run --release -- <hostname> <port>
    ```

### Example

To start the server on `localhost` with port `10000`, run:
```sh
RUST_LOG=debug cargo run --release -- localhost 10000
```
