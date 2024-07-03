//! # Chat server
//!
//! Server for simple command line chat app written in Rust.
//!
//! # Arguments:
//!
//! - **hostname** default: localhost
//! - **port** default: 11111

extern crate chat;

use anyhow::{Context, Result};
use axum::{http::StatusCode, routing::get, Router};
use env_logger::{Builder, Env};
use lazy_static::lazy_static;
use log::{debug, error, info, log_enabled, Level};
use prometheus::{Counter, Encoder, Gauge, Registry, TextEncoder};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use chat::{Message, MessageError};

const DB: &str = "sqlite://server.db";

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref MESSAGE_COUNTER: Counter =
        Counter::new("message_counter", "counts number of messages send")
            .expect("Counter metrics init failed!");
    static ref USER_COUNTER: Gauge = Gauge::new("user_counter", "counts number of connected users")
        .expect("Gauge metrics init failed!");
}

fn log_broadcasting(
    message: &Message,
    sender_addr: &std::net::SocketAddr,
    receiver_addr: &std::net::SocketAddr,
) {
    if log_enabled!(Level::Debug) {
        debug!(
            "Broadcasting message from client {:?} to client {:?} ({:?}).",
            sender_addr, receiver_addr, message
        );
    } else {
        info!(
            "Broadcasting message from client {:?} to client {:?}.",
            sender_addr, receiver_addr
        );
    };
}

fn log_incoming(message: &Message, client_addr: &std::net::SocketAddr) {
    if log_enabled!(Level::Debug) {
        debug!(
            "Incoming message from client {:?} ({:?}).",
            client_addr, message,
        );
    } else {
        info!("Incoming message from client {:?}.", client_addr);
    }
}

/// Runs the chat server.
///
/// This function initializes the database, parses the server address arguments, binds the server to the given address,
/// sets up a broadcast channel for message broadcasting, and enters a loop to accept and handle incoming client
/// connections.
///
/// # Returns
///
/// - `Result<()>`: The result of running the server. Returns `Ok(())` if successful, otherwise returns an error.
///
/// # Errors
///
/// This function will return an error if:
///
/// - There is an issue initializing the database.
/// - The server fails to bind to the specified address.
async fn run_server() -> Result<()> {
    let pool = init_db().await?;
    let address = chat::Address::parse_arguments();
    get_metrics()?;
    let listener = TcpListener::bind(address.to_string())
        .await
        .with_context(|| format!("Binding error for address: {}", address.to_string()))?;
    info!("Server listen on: {}", address.to_string());

    let (broadcast_send, _broadcast_revice) = broadcast::channel(1024);
    loop {
        let Ok((stream, addr)) = listener.accept().await else {
            error!("Failed to accept connection!");
            continue;
        };
        USER_COUNTER.inc();
        let sender = broadcast_send.clone();
        let mut receiver = broadcast_send.subscribe();
        let (mut stream_read, mut stream_writer) = stream.into_split();
        let pool_clone = pool.clone();

        tokio::spawn(async move {
            loop {
                match Message::read(&mut stream_read).await {
                    Ok(msg) => {
                        log_incoming(&msg, &addr);
                        MESSAGE_COUNTER.inc();
                        if let Err(err_msg) = insert_db(&pool_clone, &msg).await {
                            error!("Insert database error: {:?}", err_msg);
                        };
                        if sender.send((msg, addr)).is_err() {
                            break;
                        }
                    }
                    Err(MessageError::UnexpectedEof) => {
                        info!("Connection from {:?} terminated.", addr);
                        USER_COUNTER.dec();
                        break;
                    }
                    Err(err_msg) => {
                        error!("Sender Error: {:?}", err_msg);
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Ok((message, sender_addr)) = receiver.recv().await {
                if sender_addr == addr {
                    continue;
                }
                log_broadcasting(&message, &sender_addr, &addr);
                if let Err(err_msg) = message.send(&mut stream_writer).await {
                    error!("Reciever Error: {:?}", err_msg);
                    break;
                }
            }
        });
    }
}

fn logger_init() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();
}

/// Initializes the SQLite database.
///
/// This function checks if the database exists. If it does not, it creates the database.
/// It then connects to the database and ensures that the necessary tables are created.
///
/// # Returns
///
/// - `Result<SqlitePool>`: A result containing the SQLite connection pool if successful,
///   or an error if there was an issue creating or connecting to the database.
///
/// # Errors
///
/// This function will return an error if:
///
/// - There is an issue creating the database.
/// - There is an issue connecting to the database.
/// - There is an issue creating the required tables in the database.
async fn init_db() -> Result<SqlitePool> {
    if !Sqlite::database_exists(DB).await.unwrap_or(false) {
        info!("Creating database: {}", DB);
        Sqlite::create_database(DB)
            .await
            .context("Creating database error!")?;
    }
    let pool = SqlitePool::connect(DB)
        .await
        .context("Connecting database error!")?;
    create_table(&pool).await?;
    Ok(pool)
}

async fn create_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY,
        nickname TEXT NOT NULL,
        msg_type TEXT NOT NULL,
        message TEXT NOT NULL
    );
    "#,
    )
    .execute(pool)
    .await
    .context("Creating database table error!")?;
    Ok(())
}

async fn insert_db(pool: &SqlitePool, message: &Message) -> Result<()> {
    let (msg_type, message_value) = message.message.get_type_and_message();
    let mut connection = pool.acquire().await?;
    let id = sqlx::query(
        r#"
        INSERT INTO messages ( nickname, msg_type, message )
        VALUES ( ?1, ?2, ?3 )
        "#,
    )
    .bind(&message.nickname)
    .bind(msg_type)
    .bind(message_value)
    .execute(&mut *connection)
    .await
    .context("Inserting to the database error!")?
    .last_insert_rowid();
    debug!("DB insert id: {}", id);
    Ok(())
}

fn get_metrics() -> Result<()> {
    REGISTRY
        .register(Box::new(MESSAGE_COUNTER.clone()))
        .context("message counter metric registering error!")?;
    REGISTRY
        .register(Box::new(USER_COUNTER.clone()))
        .context("counter metric registering error!")?;
    Ok(())
}

async fn metrics() -> (StatusCode, String) {
    let encoder = TextEncoder::new();
    let mut buf = vec![];

    if let Err(err_msg) = encoder.encode(&REGISTRY.gather(), &mut buf) {
        error!("Metrics encoding error: {}", err_msg);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Metrics encoding error!".to_string(),
        );
    }
    if let Ok(body) = String::from_utf8(buf) {
        return (StatusCode::OK, body);
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unknow error!".to_string(),
    )
}

#[tokio::main]
async fn main() {
    logger_init();
    let app = Router::new().route("/metrics", get(metrics));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await });
    match run_server().await {
        Ok(_) => (),
        Err(err_msg) => error!("Error: {}", err_msg),
    }
}
