//! # Chat client
//!
//! ## Args:
//! - hostname [localhost]
//! - port [11111]
//!
//! ## Commands:
//!
//! - Write your message
//! - Share file: .file path_to_file.txt
//! - Share image: .image path_to_image.png
//! - Leave: .quit

extern crate chat;

use chat::Message;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{self, TcpStream};
use std::path::Path;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const IMAGE_FOLDER: &str = "IMAGES";
const FILE_FOLDER: &str = "FILES";

enum Command {
    Message(Message),
    Quit,
}

fn print_help() {
    println!("Welcome to chat!");
    println!("write your message or use command:");
    println!(".file path_to_file.txt");
    println!(".image path_to_image.png");
    println!(".quit");
    println!("");
}

fn run_client() -> Result<(), Box<dyn Error>> {
    let address = chat::parse_arguments()?;
    let stream = net::TcpStream::connect(address.to_string())?;
    let reading_stream = stream.try_clone()?;
    thread::spawn(move || {
        reading_loop(reading_stream).unwrap_or_else(|err_msg| eprintln!("{:?}", err_msg))
    });
    print_help();
    writing_loop(stream)?;
    Ok(())
}

fn reading_loop(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    loop {
        read_message(&stream)?;
    }
}

fn writing_loop(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    loop {
        match get_input() {
            Ok(result) => match result {
                Command::Quit => break,
                Command::Message(message) => send_message(&stream, message)?,
            },
            Err(err_msg) => eprintln!("{}", err_msg),
        }
    }
    Ok(())
}

fn get_input() -> Result<Command, Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();
    parse_input(input)
}

fn parse_input(input: String) -> Result<Command, Box<dyn Error>> {
    let command = if input.starts_with(".file ") {
        let (_, path) = input.split_once(" ").ok_or("Invalid command .file!")?;
        let (name, content) = get_file(path)?;
        Command::Message(Message::File { name, content })
    } else if input.starts_with(".image ") {
        let (_, path) = input.split_once(" ").ok_or("Invalid command .image!")?;
        let (_, content) = get_file(path)?;
        Command::Message(Message::Image(content))
    } else if input == ".quit" {
        Command::Quit
    } else {
        Command::Message(Message::Text(input))
    };
    Ok(command)
}

fn get_file(path: &str) -> Result<(String, Vec<u8>), Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buff = Vec::new();
    file.read_to_end(&mut buff)?;
    let name = Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("some_file")
        .to_string();
    Ok((name, buff))
}

fn send_message(mut stream: &TcpStream, message: Message) -> Result<(), Box<dyn Error>> {
    let message = message.serialized_message()?;
    let message_length = message.len() as u32;
    stream.write(&message_length.to_be_bytes())?;
    stream.write_all(&message)?;
    Ok(())
}

fn read_message(mut stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes)?;
    let message_length = u32::from_be_bytes(length_bytes) as usize;
    let mut buf = vec![0u8; message_length];
    stream.read_exact(&mut buf)?;
    let m = chat::Message::deserialized_message(&buf)?;
    handle_message(m)?;
    Ok(())
}

fn handle_message(message: Message) -> Result<(), Box<dyn Error>> {
    print!("--> ");
    match message {
        Message::Text(text) => println!("{text}"),
        Message::Image(content) => save_image(content)?,
        Message::File { name, content } => save_file(name, content)?,
    }
    Ok(())
}

fn get_timestamp() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

fn save_image(content: Vec<u8>) -> Result<(), Box<dyn Error>> {
    create_directory(IMAGE_FOLDER)?;
    let timestamp = get_timestamp()?;
    let name = format!("{timestamp:?}.png");
    let path = Path::new(IMAGE_FOLDER).join(&name);
    let mut file = File::create(path)?;
    file.write_all(&content)?;
    println!("Saving image to: {}/{}.", IMAGE_FOLDER, &name);
    Ok(())
}

fn save_file(name: String, content: Vec<u8>) -> Result<(), Box<dyn Error>> {
    create_directory(FILE_FOLDER)?;
    let path = Path::new(FILE_FOLDER).join(&name);
    let mut file = File::create(path)?;
    file.write_all(&content)?;
    println!("Saving file to: {}/{}.", FILE_FOLDER, &name);
    Ok(())
}

fn create_directory(path: &str) -> Result<(), Box<dyn Error>> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?
    }
    Ok(())
}

fn main() {
    match run_client() {
        Ok(_) => (),
        Err(err_msg) => eprintln!("Error: {}", err_msg),
    }
}
