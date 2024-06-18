//! # Chat client
//!
//! Client for simple command line chat app written in Rust.
//!
//! # Arguments:
//!
//! - **hostname** default: localhost
//! - **port** default: 11111
//!
//! # Commands:
//!
//! - Write your message
//! - Share file: .file path_to_file.txt
//! - Share image: .image path_to_image.png
//! - Leave: .quit

extern crate chat;

use chat::{Message, MessageType};
use std::path::Path;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use anyhow::{anyhow, Context, Result};
use rodio::{source::Source, Decoder, OutputStream};
use slugify::slugify;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const IMAGE_FOLDER: &str = "IMAGES";
const FILE_FOLDER: &str = "FILES";
const SOUND_FILE: &str = "meow.wav";

enum Command {
    Message(Message),
    Quit,
}

fn print_help(nickname: &str) {
    println!("");
    println!("{nickname} welcome to chat!");
    println!("");
    println!("write your message or use command:");
    println!(".file path_to_file.txt");
    println!(".image path_to_image.png");
    println!(".quit");
    println!("");
}

async fn run_client() -> Result<()> {
    let address = chat::Address::parse_arguments();
    let stream = TcpStream::connect(address.to_string()).await?;
    let (reading_stream, writing_stream) = stream.into_split();
    let nickname = get_nickname()?;
    print_help(&nickname);
    tokio::spawn(async move {
        reading_loop(reading_stream)
            .await
            .unwrap_or_else(|err_msg| eprintln!("Reading error: {:?}", err_msg))
    });
    writing_loop(writing_stream, &nickname).await?;
    Ok(())
}

fn get_nickname() -> Result<String> {
    let mut input = String::new();
    println!("Choose your nickname:");
    std::io::stdin().read_line(&mut input)?;
    let nickname = slugify!(input.trim());
    Ok(nickname)
}

async fn reading_loop(mut stream: OwnedReadHalf) -> Result<()> {
    loop {
        let message = chat::Message::read(&mut stream).await?;
        if let Err(err_msg) = handle_message(message).await {
            eprintln!("Message hadling error: {:?}", err_msg);
        };
        thread::spawn(move || {
            meow().unwrap_or_else(|err_msg| eprintln!("Sound error {:?}", err_msg))
        });
    }
}

async fn writing_loop(mut stream: OwnedWriteHalf, nickname: &str) -> Result<()> {
    loop {
        match get_input(nickname).await {
            Ok(result) => match result {
                Command::Quit => break,
                Command::Message(message) => message.send(&mut stream).await?,
            },
            Err(err_msg) => eprintln!("Input error: {}", err_msg),
        }
    }
    Ok(())
}

async fn get_input(nickname: &str) -> Result<Command> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();
    parse_input(input, nickname).await
}

async fn parse_input(input: String, nickname: &str) -> Result<Command> {
    let nickname = nickname.to_string();
    let command = if input.starts_with(".file") {
        let (_, path) = input
            .split_once(" ")
            .ok_or(anyhow!("Invalid command .file!"))?;
        let (name, content) = get_file(path).await?;
        let message = MessageType::file(name, &content);
        Command::Message(Message::from(nickname, message))
    } else if input.starts_with(".image") {
        let (_, path) = input
            .split_once(" ")
            .ok_or(anyhow!("Invalid command .image!"))?;
        let (_, content) = get_file(path).await?;
        let message = MessageType::image(&content);
        Command::Message(Message::from(nickname, message))
    } else if input == ".quit" {
        Command::Quit
    } else {
        let message = MessageType::text(input);
        Command::Message(Message::from(nickname, message))
    };
    Ok(command)
}

async fn get_file(path: &str) -> Result<(String, Vec<u8>)> {
    let mut file = File::open(path).await?;
    let mut buff = Vec::new();
    file.read_to_end(&mut buff).await?;
    let name = Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("some_file")
        .to_string();
    Ok((name, buff))
}

async fn handle_message(message: Message) -> Result<()> {
    let nickname = message.nickname;
    print!("{nickname} --> ");
    match message.message {
        MessageType::Text(text) => println!("{text}"),
        MessageType::Image(content) => save_image(content).await.context("Saving image failed!")?,
        MessageType::File { name, content } => save_file(name, content)
            .await
            .context("Saving file failed!")?,
    }
    Ok(())
}

fn meow() -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let file = std::fs::File::open(SOUND_FILE)?;
    let source = Decoder::new(std::io::BufReader::new(file))?;
    stream_handle.play_raw(source.convert_samples())?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}

fn get_timestamp() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

async fn save_image(content: Vec<u8>) -> Result<()> {
    create_directory(FILE_FOLDER).await?;
    let timestamp = get_timestamp()?;
    let name = format!("{timestamp:?}.png");
    let path = Path::new(IMAGE_FOLDER).join(&name);
    let mut file = File::create(path).await?;
    file.write_all(&content).await?;
    println!("Saving image to: {}/{}.", IMAGE_FOLDER, &name);
    Ok(())
}

async fn save_file(name: String, content: Vec<u8>) -> Result<()> {
    create_directory(FILE_FOLDER).await?;
    let path = Path::new(FILE_FOLDER).join(&name);
    let mut file = File::create(path).await?;
    file.write_all(&content).await?;
    println!("Saving file to: {}/{}.", FILE_FOLDER, &name);
    Ok(())
}

async fn create_directory(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)
            .await
            .with_context(|| format!("Creating dir {path} failed!"))?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    match run_client().await {
        Ok(_) => (),
        Err(err_msg) => eprintln!("Client error: {}", err_msg),
    }
}
