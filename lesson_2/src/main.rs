//! # transtext utility
//!
//! arguments:
//!
//! - lowercase
//! - uppercase
//! - no-spaces
//! - unchanged
//! - crabify
//! - csv

mod operations;

use operations::Operation;
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread::{self, spawn};

struct Output {
    result: String,
    operation: Operation,
}

fn parse_arguments(arguments: Vec<String>) -> Result<Operation, Box<dyn Error>> {
    let arguments_length = arguments.len();
    if arguments_length != 2 {
        return Err(From::from(format!(
            "Expecting exactly one argument, got {}!",
            arguments_length - 1
        )));
    }

    let argument: &str = &arguments[1].to_lowercase();
    Operation::from_str(argument)
}

fn get_std_input(operation: &Operation) -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    let input = match operation {
        Operation::Csv => {
            println!("Enter path to cs file to transmute:");
            let mut path = String::new();
            io::stdin().read_line(&mut path)?;
            let mut file = File::open(path.trim())?;
            file.read_to_string(&mut input)?;
            input
        }
        _ => {
            println!("Enter text to transmute:");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input
        }
    };
    Ok(input)
}

// fn transtext(arguments: Vec<String>) -> Result<Output, Box<dyn Error>> {
//     // let operation = parse_arguments(arguments)?;
//     // let input = get_std_input(&operation)?;

//     let result = match operation {
//         Operation::Lowercase => operations::lowercase(&input),
//         Operation::Uppercase => operations::uppercase(&input),
//         Operation::NoSpaces => operations::no_spaces(&input),
//         Operation::Slugify => operations::slugify(&input),
//         Operation::Unchanged => operations::unchanged(&input),
//         Operation::Crabify => operations::crabify(&input),
//         Operation::Csv => operations::csv(&input),
//     }?;

//     Ok(Output { result, operation })
// }

struct Input {
    command: Operation,
    input: String,
}

fn get_input() -> Result<Input, Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let (command, input) = input.split_once(" ").ok_or("Invalid input!")?;
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
            Err(err_msg) => eprintln!("Error: {err_msg}"),
        }
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    // let arguments: Vec<String> = env::args().collect();

    let input = thread::spawn(move || {
        handle_input(tx);
    });

    let processing = thread::spawn(move || {
        handle_command(rx);
    });

    // match transtext(arguments) {
    //     Ok(Output { result, operation }) => {
    //         eprintln!("Selected operation: {operation:?}");
    //         println!("{result}");
    //     }
    //     Err(err_msg) => eprintln!("Error: {err_msg}"),
    // }

    let _ = input.join();
    let _ = processing.join();
}
