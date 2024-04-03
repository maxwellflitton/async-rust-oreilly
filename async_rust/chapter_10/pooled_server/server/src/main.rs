use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, ErrorKind, Cursor};
use std::sync::mpsc::channel;
use std::thread;

use data_layer::data::Data;


fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_nonblocking(true)?;
    let mut buffer = Vec::new();
    let mut local_buf = [0; 1024];

    loop {
        match stream.read(&mut local_buf) {
            Ok(0) => {
                break;
            },
            Ok(len) => {
                buffer.extend_from_slice(&local_buf[..len]);
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                if buffer.len() > 0 {
                    break;
                }
                continue;
            },
            Err(e) => {
                println!("Failed to read from connection: {}", e);
            }
        }
    }
    match Data::deserialize(&mut Cursor::new(buffer.as_slice())) {
        Ok(message) => {
            println!("Received message: {:?}", message);
        },
        Err(e) => {
            println!("Failed to decode message: {}", e);
        }
    }
    stream.write_all(b"Hello, client!")?;
    Ok(())
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Server listening on port 7878");

    let (one_tx, one_rx) = channel::<TcpStream>();
    let (two_tx, two_rx) = channel::<TcpStream>();
    let (three_tx, three_rx) = channel::<TcpStream>();

    let _one = thread::spawn(move || {
        loop {
            let stream = one_rx.recv().unwrap();
            println!("One Received connection: {}", stream.peer_addr().unwrap());
            let _ = handle_client(stream);
        }
    });
    let _two = thread::spawn(move || {
        loop {
            let stream = two_rx.recv().unwrap();
            println!("Two Received connection: {}", stream.peer_addr().unwrap());
            let _ = handle_client(stream);
        }
    });
    let _three = thread::spawn(move || {
        loop {
            let stream = three_rx.recv().unwrap();
            println!("Three Received connection: {}", stream.peer_addr().unwrap());
            let _ = handle_client(stream);
        }
    });
    let router = [one_tx, two_tx, three_tx];
    let mut index = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = router[index].send(stream);
                index += 1;
                if index == 3 {
                    index = 0;
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
