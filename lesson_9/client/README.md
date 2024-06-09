# Chat Client in Rust

This is a simple chat client application written in Rust.

## Features

- **NEW** Choose a nickname at the start which will be visible to other chat participants.
- Simple command interface.
- Send and receive messages in real-time.
- Share files with other users.
- Share image files with other users.
- **NEW** Meows when a message is received.

### Notification Sound

When a new message is received, a sound will play. By default, this sound is meow.wav.
If you want to change the notification sound, replace the meow.wav file with your desired sound file.
Ensure the new file is also named meow.wav and placed in the same directory.

## Requirements

- Rust programming language installed. You can install Rust from [here](https://www.rust-lang.org/tools/install).

## Usage

### Arguments

- `hostname`: The hostname of the chat server. Default is `localhost`.
- `port`: The port of the chat server. Default is `11111`.

### Commands

- Send a message: Simply type your message and press Enter.
- Share a file: Use the command `.file path_to_file.txt` and press Enter.
- Share an image: Use the command `.image path_to_image.png` and press Enter.
- Leave the chat: Use the command `.quit` and press Enter.

### Running the Client

1. Clone the repository:
    ```sh
    git clone <repository_url>
    ```
2. Navigate to the project directory:
    ```sh
    cd <project_directory>
    ```
3. Build the project:
    ```sh
    cargo build --release
    ```
4. Run the client:
    ```sh
    cargo run --release -- <hostname> <port>
    ```

### Example

To connect to a chat server running on `localhost` with port `11111`, run:
```sh
cargo run --release -- localhost 11111
```


