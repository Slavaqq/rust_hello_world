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
use env_logger::{Builder, Env};
use log::{debug, error, info, log_enabled, Level};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use chat::{Message, MessageError};

const DB: &str = "sqlite://server.db";

fn log_broadcasting(
    message: &Message,
    sender_addr: &core::net::SocketAddr,
    receiver_addr: &core::net::SocketAddr,
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

fn log_incoming(message: &Message, client_addr: &core::net::SocketAddr) {
    if log_enabled!(Level::Debug) {
        debug!(
            "Incoming message from client {:?} ({:?}).",
            client_addr, message,
        );
    } else {
        info!("Incoming message from client {:?}.", client_addr);
    }
}

async fn run_server() -> Result<()> {
    let pool = init_db().await?;
    let address = chat::Address::parse_arguments();
    let listener = TcpListener::bind(address.to_string())
        .await
        .with_context(|| format!("Binding error for address: {}", address.to_string()))?;
    info!("Server listen on: {}", address.to_string());

    let (broadcast_send, _broadcast_revice) = broadcast::channel(1024);
    loop {
        let Ok((stream, addr)) = listener.accept().await else {
            error!("Failed to accept connection");
            continue;
        };
        let sender = broadcast_send.clone();
        let mut receiver = broadcast_send.subscribe();
        let (mut stream_read, mut stream_writer) = stream.into_split();
        let p = pool.clone();

        tokio::spawn(async move {
            loop {
                match Message::read(&mut stream_read).await {
                    Ok(msg) => {
                        log_incoming(&msg, &addr);
                        if let Err(err_msg) = insert_db(&p, &msg).await {
                            error!("Insert database error: {err_msg}");
                        };
                        if sender.send((msg, addr)).is_err() {
                            break;
                        }
                    }
                    Err(MessageError::UnexpectedEof) => {
                        info!("Connection from {:?} terminated.", addr);
                        break;
                    }
                    Err(err_msg) => {
                        error!("Sender Error: {err_msg:?}");
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
                if let Err(e) = message.send(&mut stream_writer).await {
                    error!("Reciever Error: {e}");
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

#[tokio::main]
async fn main() {
    logger_init();
    match run_server().await {
        Ok(_) => (),
        Err(err_msg) => error!("Error: {}", err_msg),
    }
}
