#[allow(dead_code)]
#[macro_use]
mod runtime;

use crate::runtime::{FutureType, spawn_task_function};
#[allow(dead_code)]
mod hyper_client;

use runtime::Runtime;

use futures_lite::future;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll as MioPoll, Token};
use std::io::{Read, Write};
use std::time::Duration;
use std::error::Error;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::io::ErrorKind;

const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);


struct ServerFuture {
    server: TcpListener,
    poll: MioPoll,
}


impl Future for ServerFuture {

    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {

        let mut events = Events::with_capacity(1);

        let _ = self.poll.poll(
            &mut events,
            Some(Duration::from_millis(200))
        ).unwrap();

        for event in events.iter() {
            if event.token() == SERVER && event.is_readable() {
                let (mut stream, _) = self.server.accept().unwrap();

                let mut buffer = [0u8; 1024];
                
                let mut received_data = Vec::new();
                
                loop {
                    match stream.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            received_data.extend_from_slice(&buffer[..n]);
                        }
                        Ok(_) => {
                            break;
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // If the stream would block, return Poll::Pending and set the waker.
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        Err(e) => {
                            eprintln!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                }
                if !received_data.is_empty() {
                    let received_str = String::from_utf8_lossy(&received_data);
                    return Poll::Ready(received_str.to_string())
                }
                cx.waker().wake_by_ref();
                return Poll::Pending
            }
        }
        cx.waker().wake_by_ref();
        return Poll::Pending
    }
}


pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    pub fn receive(&self) -> T {
        let mut b = self.queue.lock().unwrap();
        loop {
            if let Some(message) = b.pop_front() {
                return message;
            }
            b = self.item_ready.wait(b).unwrap();
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    Runtime::new().with_low_num(2).with_high_num(4).run();

    let addr = "127.0.0.1:13265".parse()?;
    let mut server = TcpListener::bind(addr)?;
    let mut stream = TcpStream::connect(server.local_addr()?)?;

    let poll: MioPoll = MioPoll::new()?;
    poll.registry()
    .register(&mut server, SERVER, Interest::READABLE)?;

    let server_worker = ServerFuture{
        server,
        poll,
    };
    let test = spawn_task!(server_worker);

    let mut client_poll: MioPoll = MioPoll::new()?;
    client_poll.registry()
    .register(&mut stream, CLIENT, Interest::WRITABLE)?;

    let mut events = Events::with_capacity(128);

    let _ = client_poll.poll(
        &mut events,
        None
    ).unwrap();

    for event in events.iter() {
        if event.token() == CLIENT && event.is_writable() {
            let message = "that's so dingo!\n";
            let _ = stream.write_all(message.as_bytes());
        }
    }

    let outcome = future::block_on(test);
    println!("outcome: {}", outcome);

    Ok(())

}
