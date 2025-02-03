# Trainspotter

Trainspotter is a Rust project that connects to the National Rail push port system using the STOMP protocol. It provides an asynchronous client for subscribing to live train data and handling incoming messages, all whilst abstracting away the low-level protocol details.

## Features

* Asynchronous I/O: Utilises Tokio for non-blocking TCP connections
* STOMP Protocol: Handles connection, subscription, and message parsing with minimal setup
* Gzipped Data Support: Automatically decompresses gzipped message bodies using flate2
* Custom Message Handling: Allows you to define your own callback to process each received message

## Installation

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
flate2 = "1.0"
```

## Usage

Below is a simple example demonstrating how to use Trainspotter to connect to a STOMP server, subscribe to a topic, and process incoming messages:

```rust
use trainspotter::NationalRailPushPortClient;
use tokio;

#[tokio::main]
async fn main() {
    // Replace with your server's details.
    let host = "darwin-dist-44ae45.nationalrail.co.uk";
    let port = 61613;
    let username = "your_username";
    let password = "your_password";
    let topic = "liveFeed";

    // Connect to the STOMP server.
    let mut client = NationalRailPushPortClient::connect(host, port, username, password)
        .await
        .expect("Failed to connect to the STOMP server");

    // Subscribe to a topic.
    client.subscribe(topic)
        .await
        .expect("Failed to subscribe to topic");

    // Read and handle messages.
    client.read_messages(|message| {
        println!("Received message: {}", message);
        Ok(())
    })
    .await
    .expect("Error reading messages");
}
```

## Licence

This project is licensed under the MIT Licence.

## Contributing

Contributions are welcome! If you have suggestions or improvements, please open an issue or submit a pull request.

Happy trainspotting!