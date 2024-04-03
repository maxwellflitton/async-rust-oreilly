use std::net::TcpStream;
use std::io::{self, Read, Write};
use data_layer::data::Data;


fn main() -> io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    // Create a new DataMessage
    let message = Data {
        field1: 1,
        field2: 2,
        field3: "Hello, server!".to_string(),
    };
    stream.write_all(&message.serialize()?)?;

    // Wait for the server to echo the data back
    let mut buffer = [0; 1024];
    println!("Reading...");
    let n = stream.read(&mut buffer)?;

    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
    Ok(())
}
