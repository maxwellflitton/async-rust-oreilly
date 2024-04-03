use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// A trait for our custom future behavior
trait Logging {
    fn log(&self);
}

// A decorator Future that adds logging behavior
struct LoggingFuture<F: Future + Logging> {
    inner: F,
}

impl<F: Future + Logging> Future for LoggingFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: we're not moving out of the inner future, just projecting through the Pin.
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.log(); // Log the polling action
        inner.poll(cx) // Delegate to the inner future's poll method
    }
}

// Implement the Logging trait for any type that also implements Future
impl<F: Future> Logging for F {
    fn log(&self) {
        println!("Polling the future!");
    }
}

// Example usage with an actual future
async fn my_async_function() -> String {
    // Simulate some async work
    "Result of async computation".to_string()
}

#[tokio::main]
async fn main() {
    let logged_future = LoggingFuture { inner: my_async_function() };

    // Normally, you would await the future, but here we just want to show the logging behavior.
    let result = logged_future.await;
    println!("{}", result);
}
