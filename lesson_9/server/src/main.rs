//! # Chat server
//!
//! ## Args:
//! - hostname [localhost]
//! - port [11111]
extern crate chat;

use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{self, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<HashMap<usize, TcpStream>>>;

fn broadcasting(
    sender_id: &usize,
    clients: &Clients,
    message: &[u8],
) -> Result<(), Box<dyn Error>> {
    let clients = clients.lock().unwrap();

    for (client_id, stream) in clients.iter() {
        if client_id != sender_id {
            let mut stream = stream.try_clone()?;
            stream.write_all(message)?;
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
        let mut clients = clients.lock().unwrap();
        clients.insert(client_id, stream.try_clone()?);
    }

    println!("Client {} connects!", client_id);

    let mut read_stream = stream.try_clone()?;

    loop {
        let bytes_read = read_stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        let message = &buf[0..bytes_read];
        broadcasting(&client_id, &clients, message)?;
    }

    {
        let mut clients = clients.lock().unwrap();
        clients.remove(&client_id);
    }

    println!("Client {} leaves!", client_id);

    Ok(())
}

fn run_server() -> Result<(), Box<dyn Error>> {
    let address = chat::parse_arguments()?;
    println!("{:?}", address);
    let listener = net::TcpListener::bind(address.to_string())?;
    eprintln!("Server listen on: {}", address.to_string());

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let mut client_id: usize = 0;

    for stream in listener.incoming() {
        let stream = stream?;
        let clients = Arc::clone(&clients);
        client_id += 1;

        thread::spawn(move || {
            handle_client(client_id, clients, stream)
                .unwrap_or_else(|err_msg| eprintln!("{:?}", err_msg))
        });
    }
    Ok(())
}

fn main() {
    match run_server() {
        Ok(_) => (),
        Err(err_msg) => eprintln!("Error: {}", err_msg),
    }
}
