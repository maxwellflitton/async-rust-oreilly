use std::{
    future::Future,
    task::{Context, Poll},
    pin::Pin,
    net::TcpStream,
    io::{self, Write},
    sync::{Arc, Mutex}
};


pub struct TcpSender {
    pub stream: Arc<Mutex<TcpStream>>,
    pub buffer: Vec<u8>
}

impl Future for TcpSender {

    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut stream = match self.stream.try_lock() {
            Ok(stream) => stream,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };
        stream.set_nonblocking(true)?;
        match stream.write_all(&self.buffer) {
            Ok(_) => {
                Poll::Ready(Ok(()))
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // The operation would block, so we need to try again later.
                // Register the current task to be woken up once the stream is readable.
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            Err(e) => Poll::Ready(Err(e)), // Some other error occurred
        }
    }
}