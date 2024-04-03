use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, ErrorKind};
use std::thread;
use data_layer::data::DataMessage;
use prost::Message;


fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_nonblocking(true)?;
    let mut buffer = Vec::new();
    let mut local_buf = [0; 1024];

    loop {
        println!("Reading...");
        match stream.read(&mut local_buf) {
            Ok(0) => {
                println!("Connection closed");
                break;
            },
            Ok(len) => {
                println!("Read {} bytes", len);
                buffer.extend_from_slice(&local_buf[..len]);
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                println!("No data left to read");
                break
            },
            Err(e) => {
                println!("Failed to read from connection: {}", e);
            }
        }
    }
    if let Ok(message) = DataMessage::decode(buffer.as_slice()) {
        println!("Received message: {:?}", message);
    } else {
        eprintln!("Failed to decode message.");
    }
    stream.write_all(b"Hello, client!")?;
    Ok(())
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Server listening on port 7878");

    // Handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                // Spawn a new thread to handle the connection
                thread::spawn(move || {
                    let _ = handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
