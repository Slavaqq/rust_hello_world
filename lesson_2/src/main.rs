//! # transtext utility
//!
//! arguments:
//!
//! - lowercase
//! - uppercase
//! - no-spaces
//! - unchanged
//! - crabify

use slug::slugify;
use std::env;
use std::io;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() != 2 {
        panic!("Expecting exactly one argument!")
    }

    let argument: &str = &arguments[1].to_lowercase();

    let transtext_func = match argument {
        "lowercase" => |x: &str| x.to_lowercase(),
        "uppercase" => |x: &str| x.to_uppercase(),
        "no-spaces" => |x: &str| x.replace(" ", ""),
        "slugify" => |x: &str| slugify(x),
        "unchanged" => |x: &str| String::from(x),
        "crabify" => |x: &str| "🦀".repeat(x.chars().count()),
        _ => {
            panic!("Unknown argument!")
        }
    };
    println!("Enter text to transmute:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Read line failed!");

    let input = input.trim();
    let output = transtext_func(input);
    println!("{output}")
}
