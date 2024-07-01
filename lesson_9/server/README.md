# Chat Server in Rust

This is a simple chat server application written in Rust. It allows multiple clients to connect and communicate with each other in real-time.

## Features

- Accept multiple client connections.
- Broadcast messages from one client to all other connected clients.
- [use `parking_lot::Mutex`](https://crates.io/crates/parking_lot)
- [use `env_logger` for logging](https://crates.io/crates/env_logger)
- [use `tokio` for async](https://crates.io/crates/tokio)
- [use `sqlx` for handling database](https://crates.io/crates/sqlx)
- [**NEW** use `rocket` for web admin panel](https://crates.io/crates/rocket)

## Requirements

- Rust programming language installed. You can install Rust from [here](https://www.rust-lang.org/tools/install).


## Admin Panel

Web interface for admin operation like show or delete messages from database.

## Database

There is SQLite database `server.db` holding message data. Check the databse content with:

```sh
sqlite3 server.db "SELECT * FROM messages;"
```

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
    cargo build --release --bin server
    ```
4. Run the server:
    ```sh
    cargo run --release --bin server -- <hostname> <port>
    ```

### Example

To start the server on `localhost` with port `10000`, run:
```sh
RUST_LOG=debug cargo run --bin server --release -- localhost 10000
```

### Running the Admin Panel

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
    cargo build --release --bin admin
    ```
4. Run the server:
    ```sh
    cargo run --release --bin admin
    ```

5. Open browser `http://127.0.0.1:8000`
