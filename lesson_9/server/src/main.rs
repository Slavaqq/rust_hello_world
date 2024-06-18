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
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use chat::{Message, MessageError};

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

        tokio::spawn(async move {
            loop {
                match Message::read(&mut stream_read).await {
                    Ok(msg) => {
                        log_incoming(&msg, &addr);
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

#[tokio::main]
async fn main() {
    logger_init();
    match run_server().await {
        Ok(_) => (),
        Err(err_msg) => error!("Error: {}", err_msg),
    }
}
