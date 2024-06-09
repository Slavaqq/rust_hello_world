use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

const HOSTNAME: &str = "localhost";
const PORT: &str = "11111";

/// Represents the address of the server with hostname and port.
#[derive(Debug)]
pub struct Address {
    hostname: String,
    port: String,
}

/// Represents a message with a nickname and a message type.
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub nickname: String,
    pub message: MessageType,
}

/// Enum representing different types of messages.
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    /// Text message.
    Text(String),
    // Image message with a vector of bytes.
    Image(Vec<u8>),
    /// File message with a name and content as a vector of bytes.
    File {
        name: String,
        content: Vec<u8>,
    },
}

impl Address {
    /// Creates a new Address with the specified hostname and port.
    ///
    /// # Arguments
    ///
    /// - `hostname` - A string slice that holds the hostname.
    /// - `port` - A string slice that holds the port.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::Address;
    /// let addr = Address::new("0.0.0.0".to_string(), "10000".to_string());
    /// assert_eq!(addr.to_string(), "0.0.0.0:10000");
    /// ```
    pub fn new(hostname: String, port: String) -> Address {
        Address { hostname, port }
    }

    /// Creates a default Address using the constants HOSTNAME and PORT.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::Address;
    /// let addr = Address::default();
    /// assert_eq!(addr.to_string(), "localhost:11111");
    /// ```
    pub fn default() -> Address {
        Address {
            hostname: HOSTNAME.to_string(),
            port: PORT.to_string(),
        }
    }
}

impl ToString for Address {
    /// Converts the Address to a string in the format "hostname:port".
    ///
    /// # Example
    ///
    /// ```
    /// use chat::Address;
    /// let addr = Address::new("localhost".to_string(), "11111".to_string());
    /// assert_eq!(addr.to_string(), "localhost:11111")
    /// ```
    fn to_string(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }
}

/// Parses command-line arguments to create an Address.
///
/// If the correct number of arguments is not provided, it returns a default Address.
///
/// # Returns
///
/// - `Ok(Address)` - If parsing is successful.
/// - `Err(Box<dyn Error>)` - If an error occurs during parsing.
///
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
    /// Serializes the Message to a vector of bytes.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<u8>)` - If serialization is successful.
    /// - `Err(Box<dyn Error>)` - If an error occurs during serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::{Message, MessageType};
    /// let msg = Message { nickname: "user".to_string(), message: MessageType::Text("Hello".to_string()) };
    /// let serialized_msg = msg.serialized_message().unwrap();
    /// let msg_bytes: Vec<u8> = vec![4, 0, 0, 0, 0, 0, 0, 0, 117, 115, 101, 114, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111];
    /// assert_eq!(serialized_msg, msg_bytes);
    /// ```
    pub fn serialized_message(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let serialized = bincode::serialize(&self)?;
        Ok(serialized)
    }
    /// Deserializes a vector of bytes to a Message.
    ///
    /// # Arguments
    ///
    /// - `input` - A byte slice that holds the serialized message.
    ///
    /// # Returns
    ///
    /// - `Ok(Message)` - If deserialization is successful.
    /// - `Err(Box<dyn Error>)` - If an error occurs during deserialization.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::{Message, MessageType};
    /// let bytes: Vec<u8> = vec![4, 0, 0, 0, 0, 0, 0, 0, 117, 115, 101, 114, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111];
    /// let deserialized_msg = Message::deserialized_message(&bytes).unwrap();
    /// let msg = Message { nickname: "user".to_string(), message: MessageType::Text("Hello".to_string()) };
    /// assert_eq!(deserialized_msg.nickname, msg.nickname);
    /// ```
    pub fn deserialized_message(input: &[u8]) -> Result<Message, Box<dyn Error>> {
        let deserialized = bincode::deserialize(input)?;
        Ok(deserialized)
    }
}
