//! # transtext utility
//!
//! Commands:
//!
//! - lowercase
//! - uppercase
//! - no-spaces
//! - unchanged
//! - crabify
//! - csv

mod operations;

use operations::Operation;
use std::error::Error;
use std::io;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

struct Output {
    result: String,
    operation: Operation,
}

struct Input {
    command: Operation,
    input: String,
}

fn get_input() -> Result<Input, Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let (command, input) = input.split_once(" ").ok_or("Invalid <command> <input>!")?;
    let command = Operation::from_str(command)?;
    let input = input.to_string();

    Ok(Input { command, input })
}

fn handle_input(tx: mpsc::Sender<Input>) {
    loop {
        println!("Enter <command> <input>:");
        match get_input() {
            Ok(input) => {
                if tx.send(input).is_err() {
                    eprintln!("Unable to send input!");
                    break;
                }
            }
            Err(err_msg) => eprintln!("Interactive input Error: {}", err_msg),
        }
    }
}

fn transtext(rx: &mpsc::Receiver<Input>) -> Result<Output, Box<dyn Error>> {
    let received = rx.recv()?;
    let result = match received.command {
        Operation::Lowercase => operations::lowercase(&received.input),
        Operation::Uppercase => operations::uppercase(&received.input),
        Operation::NoSpaces => operations::no_spaces(&received.input),
        Operation::Slugify => operations::slugify(&received.input),
        Operation::Unchanged => operations::unchanged(&received.input),
        Operation::Crabify => operations::crabify(&received.input),
        Operation::Csv => operations::csv(&received.input),
    }?;

    Ok(Output {
        result,
        operation: received.command,
    })
}

fn handle_command(rx: mpsc::Receiver<Input>) {
    loop {
        match transtext(&rx) {
            Ok(Output { result, operation }) => {
                eprintln!("Selected operation: {operation:?}");
                println!("{result}");
            }
            Err(err_msg) => eprintln!("Processing Error: {err_msg}"),
        }
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();

    let input = thread::spawn(move || {
        handle_input(tx);
    });

    let processing = thread::spawn(move || {
        handle_command(rx);
    });

    let _ = input.join();
    let _ = processing.join();
}
