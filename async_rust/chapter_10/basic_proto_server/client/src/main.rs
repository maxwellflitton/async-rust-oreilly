use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use data_layer::data::DataMessage;
use prost::Message;


#[tokio::main]
async fn main() -> io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;

    // Create a new DataMessage
    let message = DataMessage {
        message: "Hello, server!".to_string(),
    };

    // Serialize the DataMessage to a Vec<u8>
    let mut buf = Vec::new();
    message.encode(&mut buf)?;
    stream.write_all(&buf).await?;

    // Send some data to the server
    // stream.write_all(b"Hello, server!").await?;

    // Wait for the server to echo the data back
    let mut buffer = [0; 1024];
    println!("Reading...");
    let n = stream.read(&mut buffer).await?;

    // Print the data received from the server
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
