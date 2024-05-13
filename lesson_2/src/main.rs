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
use std::io::{self, Read};

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
    match argument {
        "lowercase" => Ok(Operation::Lowercase),
        "uppercase" => Ok(Operation::Uppercase),
        "no-spaces" => Ok(Operation::NoSpaces),
        "slugify" => Ok(Operation::Slugify),
        "unchanged" => Ok(Operation::Unchanged),
        "crabify" => Ok(Operation::Crabify),
        "csv" => Ok(Operation::Csv),
        _ => Err(From::from(format!("Unknown argument: {argument}!"))),
    }
}

fn get_std_input(operation: &Operation) -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    let input = match operation {
        Operation::Csv => {
            println!("Enter csv to transmute (Press Ctrl-D to end):");
            io::stdin().read_to_string(&mut input)?;
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

fn transtext(arguments: Vec<String>) -> Result<Output, Box<dyn Error>> {
    let operation = parse_arguments(arguments)?;
    let input = get_std_input(&operation)?;

    let result = match operation {
        Operation::Lowercase => operations::lowercase(&input),
        Operation::Uppercase => operations::uppercase(&input),
        Operation::NoSpaces => operations::no_spaces(&input),
        Operation::Slugify => operations::slugify(&input),
        Operation::Unchanged => operations::unchanged(&input),
        Operation::Crabify => operations::crabify(&input),
        Operation::Csv => operations::csv(&input),
    }?;

    Ok(Output { result, operation })
}

fn main() {
    let arguments: Vec<String> = env::args().collect();

    match transtext(arguments) {
        Ok(Output { result, operation }) => {
            eprintln!("Selected operation: {operation:?}");
            println!("{result}");
        }
        Err(err_msg) => eprintln!("Error: {err_msg}"),
    }
}
