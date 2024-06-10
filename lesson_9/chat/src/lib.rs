use std::io::{Read, Write};
use std::net::TcpStream;
use std::{env, io};

use bincode::Error as BincodeError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("de/serialization error")]
    DeSerializationError(#[from] BincodeError),
    #[error(transparent)]
    IOError(#[from] io::Error),
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
    /// Parses command-line arguments to create an Address.
    ///
    /// If the correct number of arguments is not provided, it returns a default Address.
    ///
    /// # Returns
    ///
    /// - `Ok(Address)` - If parsing is successful.
    /// - `Err(Box<dyn Error>)` - If an error occurs during parsing.
    ///
    pub fn parse_arguments() -> Address {
        let arguments: Vec<String> = env::args().collect();

        match arguments.len() {
            3 => Address::new(
                arguments.get(1).unwrap_or(&HOSTNAME.into()).clone(),
                arguments.get(2).unwrap_or(&PORT.into()).clone(),
            ),
            _ => Address::default(),
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

impl MessageType {
    /// Creates a Text type MessageType.
    ///
    /// # Arguments
    ///
    /// - `text` - A string slice that holds the text of the message.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::MessageType;
    /// let msg = MessageType::text("Hello");
    /// ```
    pub fn text<S: AsRef<str>>(text: S) -> Self {
        MessageType::Text(text.as_ref().into())
    }

    /// Creates a Text type MessageType.
    ///
    /// # Arguments
    ///
    /// - `name` - A string slice that holds the name.
    /// - `data` - File content.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::MessageType;
    /// let file_data = vec![0u8; 10];
    /// let msg = MessageType::file("test.txt", &file_data);
    /// ```
    pub fn file<S: AsRef<str>>(name: S, data: &[u8]) -> Self {
        MessageType::File {
            name: name.as_ref().into(),
            content: data.to_vec(),
        }
    }
    /// Creates a Text type MessageType.
    ///
    /// # Arguments
    ///
    /// - `data` - File content.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::MessageType;
    /// let file_data = vec![0u8; 10];
    /// let msg = MessageType::image(&file_data);
    /// ```
    pub fn image(data: &[u8]) -> Self {
        MessageType::Image(data.to_vec())
    }
}

impl Message {
    /// Creates a new Message with the specified nickname and Message.
    ///
    /// # Arguments
    ///
    /// - `nickaname` - A string slice that holds the nickname.
    /// - `message` - A MessageType.
    ///
    /// # Example
    ///
    /// ```
    /// use chat::Message;
    /// use chat::MessageType;
    /// let m = MessageType::text("Hello");
    /// let msg = Message::from("user", m);
    /// assert_eq!(msg.nickname, "user");
    /// ```
    pub fn from<S: AsRef<str>>(nickname: S, message: MessageType) -> Self {
        Message {
            nickname: nickname.as_ref().into(),
            message,
        }
    }

    /// Send a Message over the TcpStream.
    ///
    ///
    /// # Arguments
    ///
    /// - `stream` - mutable TcpStream.
    ///
    pub fn send(&self, mut stream: &TcpStream) -> Result<(), MessageError> {
        let message = self.serialized_message()?;
        let message_length = message.len() as u32;
        let mut full_message = message_length.to_be_bytes().to_vec();
        full_message.extend(message);
        stream.write_all(&full_message)?;
        Ok(())
    }

    /// Read a Message from the TcpStream.
    ///
    ///
    /// # Arguments
    ///
    /// - `stream` - mutable TcpStream.
    ///
    pub fn read(mut stream: &TcpStream) -> Result<Self, MessageError> {
        let mut length_bytes = [0u8; 4];
        stream.read_exact(&mut length_bytes)?;
        let message_length = u32::from_be_bytes(length_bytes) as usize;
        let mut buf = vec![0u8; message_length];
        stream.read_exact(&mut buf)?;
        Ok(Message::deserialized_message(&buf)?)
    }
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
    pub fn serialized_message(&self) -> Result<Vec<u8>, BincodeError> {
        bincode::serialize(&self)
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
    pub fn deserialized_message(input: &[u8]) -> Result<Message, BincodeError> {
        bincode::deserialize(input)
    }
}
