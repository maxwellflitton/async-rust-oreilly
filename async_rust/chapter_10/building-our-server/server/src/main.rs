use std::{
    thread,
    sync::{mpsc::channel, atomic::{AtomicBool, Ordering}},
    io::{self, Read, Write, ErrorKind, Cursor},
    net::{TcpListener, TcpStream}
};
use data_layer::data::Data;
use async_runtime::{
    executor::Executor,
    sleep::Sleep
};


static FLAGS: [AtomicBool; 3] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];


macro_rules! spawn_worker {
    ($name:expr, $rx:expr, $flag:expr) => {
        thread::spawn(move || {
            let mut executor = Executor::new();
            loop {
                if let Ok(stream) = $rx.try_recv() {
                    println!(
                        "{} Received connection: {}", 
                        $name, 
                        stream.peer_addr().unwrap()
                    );
                    executor.spawn(handle_client(stream));
                } else {
                    if executor.polling.len() == 0 {
                        println!("{} is sleeping", $name);
                        $flag.store(true, Ordering::SeqCst);
                        thread::park();
                    }
                }
                executor.poll();
            }
        })
    };
}


async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
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
                println!("starting to sleep");
                Sleep::new(std::time::Duration::from_millis(10)).await;
                println!("finished sleeping");
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
    Sleep::new(std::time::Duration::from_secs(1)).await;
    stream.write_all(b"Hello, client!")?;
    Ok(())
}



fn main() -> io::Result<()> {
    let (one_tx, one_rx) = channel::<TcpStream>();
    let (two_tx, two_rx) = channel::<TcpStream>();
    let (three_tx, three_rx) = channel::<TcpStream>();

    let one = spawn_worker!("One", one_rx, &FLAGS[0]);
    let two = spawn_worker!("Two", two_rx, &FLAGS[1]);
    let three = spawn_worker!("Three", three_rx, &FLAGS[2]);

    let router = [one_tx, two_tx, three_tx];
    let threads = [one, two, three];
    let mut index = 0;

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = router[index].send(stream);
                if FLAGS[index].load(Ordering::SeqCst) {
                    FLAGS[index].store(false, Ordering::SeqCst);
                    threads[index].thread().unpark();
                }
                index += 1;  // cycle through the index of threads
                if index == 3 {
                    index = 0;
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}
