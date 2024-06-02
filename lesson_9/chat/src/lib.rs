use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

const HOSTNAME: &str = "localhost";
const PORT: &str = "11111";

#[derive(Debug)]
pub struct Address {
    hostname: String,
    port: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Text(String),
    Image(Vec<u8>),
    File { name: String, content: Vec<u8> },
}

impl Address {
    fn new(hostname: String, port: String) -> Address {
        Address { hostname, port }
    }

    fn default() -> Address {
        Address {
            hostname: HOSTNAME.to_string(),
            port: PORT.to_string(),
        }
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }
}

pub fn parse_arguments() -> Result<Address, Box<dyn Error>> {
    let arguments: Vec<String> = env::args().collect();

    match arguments.len() {
        3 => Ok(Address::new(
            arguments.get(1).unwrap_or(&HOSTNAME.to_string()).clone(),
            arguments.get(2).unwrap_or(&PORT.to_string()).clone(),
        )),
        _ => Ok(Address::default()),
    }
}

impl Message {
    pub fn serialized_message(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let serialized = bincode::serialize(&self)?;
        Ok(serialized)
    }

    pub fn deserialized_message(input: &[u8]) -> Result<Message, Box<dyn Error>> {
        let deserialized = bincode::deserialize(input)?;
        Ok(deserialized)
    }
}
