use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::error::Error;

use crate::frame::{parse_stomp_frame, decompress_gzipped_data};

/// A client for connecting to National Rails push port system.
pub struct NationalRailPushPortClient {
    stream: TcpStream,
    accumulated: Vec<u8>,
}

impl NationalRailPushPortClient {
    /// Connects to a STOMP server and performs the initial handshake.
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let address = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(address).await?;
        println!("Connected to STOMP server.");

        // Send the CONNECT frame.
        let connect_frame = format!(
            "CONNECT\naccept-version:1.2\nhost:{}\nlogin:{}\npasscode:{}\n\n\0",
            host, username, password
        );
        stream.write_all(connect_frame.as_bytes()).await?;
        println!("Sent CONNECT frame:\n{}", connect_frame);

        // Read the server's CONNECTED response.
        let mut buffer = vec![0u8; 8192];
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            return Err("No response received. Connection may have been closed.".into());
        }
        println!("Received response:\n{}", String::from_utf8_lossy(&buffer[..n]));

        Ok(Self {
            stream,
            accumulated: Vec::new(),
        })
    }

    /// Sends a frame to the server.
    pub async fn send_frame(&mut self, frame: &str) -> Result<(), Box<dyn Error>> {
        self.stream.write_all(frame.as_bytes()).await?;
        Ok(())
    }

    /// Subscribes to a given topic.
    pub async fn subscribe(&mut self, live_feed_topic: &str) -> Result<(), Box<dyn Error>> {
        let subscribe_frame = format!(
            "SUBSCRIBE\nid:sub-1\ndestination:/topic/{}\nack:auto\n\n\0",
            live_feed_topic
        );
        self.send_frame(&subscribe_frame).await?;
        println!("Sent SUBSCRIBE frame:\n{}", subscribe_frame);
        Ok(())
    }

    /// Reads data from the connection, processes complete STOMP frames, and calls a provided callback with the message string.
    ///
    /// The callback receives the decompressed message (or the raw body if decompression fails).
    pub async fn read_messages<F>(&mut self, mut message_callback: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(String) -> Result<(), Box<dyn Error>>,
    {
        loop {
            let mut buf = vec![0u8; 4096];
            let n = self.stream.read(&mut buf).await?;
            if n == 0 {
                println!("Connection closed by server.");
                break;
            }
            self.accumulated.extend_from_slice(&buf[..n]);

            // Process every complete frame available.
            while let Some((frame_len, frame)) = parse_stomp_frame(&self.accumulated) {
                // Remove the processed frame from the accumulator.
                self.accumulated.drain(..frame_len);

                // Process the frame body:
                let message = if frame.body.is_empty() {
                    // No body means an empty message.
                    String::new()
                } else {
                    // Attempt to decompress the body.
                    match decompress_gzipped_data(&frame.body) {
                        Ok(decompressed) => decompressed,
                        Err(_) => {
                            // If decompression fails, fallback to treating the body as plain text.
                            String::from_utf8_lossy(&frame.body).to_string()
                        }
                    }
                };
                message_callback(message)?;
            }
        }
        Ok(())
    }
}
