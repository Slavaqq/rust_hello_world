//! # Chat server
//!
//! Server for simple command line chat app written in Rust.
//!
//! # Arguments:
//!
//! - **hostname** default: localhost
//! - **port** default: 11111

extern crate chat;

use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{self, TcpStream};
use std::sync::Arc;
use std::thread;

use env_logger::{Builder, Env};
use log::{debug, error, info, log_enabled, Level};
use parking_lot::Mutex;

type Clients = Arc<Mutex<HashMap<usize, TcpStream>>>;

fn log_broadcasting(
    stream: &TcpStream,
    message: &[u8],
    sender_id: &usize,
    client_id: &usize,
) -> Result<(), Box<dyn Error>> {
    if log_enabled!(Level::Debug) {
        let addr = stream.peer_addr()?;
        debug!(
            "Broadcasting message (length {}) from client {}, to client {} on addr: {}",
            &message.len(),
            &sender_id,
            &client_id,
            addr
        );
    } else {
        info!(
            "Broadcasting message from client {} to client {}",
            &sender_id, &client_id
        );
    }
    Ok(())
}

fn log_incoming(message: &[u8], client_id: &usize) {
    if log_enabled!(Level::Debug) {
        debug!(
            "Incomig message (length {}) from client {}.",
            &message.len(),
            &client_id
        );
    } else {
        info!("Incoming message from client {}.", &client_id);
    }
}

fn broadcasting(
    sender_id: &usize,
    clients: &Clients,
    message: &[u8],
) -> Result<(), Box<dyn Error>> {
    let clients = clients.lock();
    for (client_id, stream) in clients.iter() {
        if client_id != sender_id {
            let mut stream = stream.try_clone()?;
            stream.write_all(message)?;
            log_broadcasting(&stream, message, sender_id, client_id)?;
        }
    }
    Ok(())
}

fn handle_client(
    client_id: usize,
    clients: Clients,
    stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 128];
    {
        let mut clients = clients.lock();
        clients.insert(client_id, stream.try_clone()?);
    }
    info!("Client {} connects!", client_id);
    let mut read_stream = stream.try_clone()?;

    loop {
        let bytes_read = read_stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        let message = &buf[0..bytes_read];
        log_incoming(&message, &client_id);
        broadcasting(&client_id, &clients, message)?;
    }

    {
        let mut clients = clients.lock();
        clients.remove(&client_id);
    }
    info!("Client {} leaves!", client_id);
    Ok(())
}

fn run_server() -> Result<(), Box<dyn Error>> {
    let address = chat::parse_arguments()?;
    let listener = net::TcpListener::bind(address.to_string())?;
    info!("Server listen on: {}", address.to_string());

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let mut client_id: usize = 0;

    for stream in listener.incoming() {
        let stream = stream?;
        let clients = Arc::clone(&clients);
        client_id += 1;

        thread::spawn(move || {
            handle_client(client_id, clients, stream)
                .unwrap_or_else(|err_msg| error!("{:?}", err_msg))
        });
    }
    Ok(())
}

fn logger_init() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();
}

fn main() {
    logger_init();
    match run_server() {
        Ok(_) => (),
        Err(err_msg) => error!("Error: {}", err_msg),
    }
}
