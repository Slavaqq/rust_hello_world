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

use std::env;
use std::error::Error;
use std::io::{self, Read};

mod operations;

struct Output {
    result: String,
    operation: operations::Operation,
}

fn parse_arguments(arguments: Vec<String>) -> Result<operations::Operation, Box<dyn Error>> {
    let arguments_length = arguments.len();
    if arguments_length != 2 {
        return Err(From::from(format!(
            "Expecting exactly one argument, got {}!",
            arguments_length - 1
        )));
    }

    let argument: &str = &arguments[1].to_lowercase();
    match argument {
        "lowercase" => Ok(operations::Operation::Lowercase),
        "uppercase" => Ok(operations::Operation::Uppercase),
        "no-spaces" => Ok(operations::Operation::NoSpaces),
        "slugify" => Ok(operations::Operation::Slugify),
        "unchanged" => Ok(operations::Operation::Unchanged),
        "crabify" => Ok(operations::Operation::Crabify),
        "csv" => Ok(operations::Operation::Csv),
        _ => Err(From::from(format!("Unknown argument: {argument}!"))),
    }
}

fn get_std_input(operation: &operations::Operation) -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    let input = match operation {
        operations::Operation::Csv => {
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
        operations::Operation::Lowercase => operations::lowercase(&input),
        operations::Operation::Uppercase => operations::uppercase(&input),
        operations::Operation::NoSpaces => operations::no_spaces(&input),
        operations::Operation::Slugify => operations::slugify(&input),
        operations::Operation::Unchanged => operations::unchanged(&input),
        operations::Operation::Crabify => operations::crabify(&input),
        operations::Operation::Csv => operations::csv(&input),
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
