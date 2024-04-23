use std::env;
use std::io;

fn main() {
    let mut arguments = env::args();
    let _ = arguments.next().unwrap();
    let greeting = arguments.next().unwrap_or("Hello".to_string());

    println!("Enter your name:");
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Read line failed!");
    let name = name.trim();

    println! {"{greeting} {name}!"}
}
