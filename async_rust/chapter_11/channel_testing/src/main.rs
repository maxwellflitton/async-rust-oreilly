
fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;
    use tokio::time::{Duration, timeout};
    use tokio::runtime::Builder;

    #[test]
    fn test_channel_capacity() {
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let (sender, mut receiver) = mpsc::channel::<i32>(5);

        let receiver = runtime.spawn(async move {
            let mut i = 0;
            while let Some(msg) = receiver.recv().await {
                assert_eq!(msg, i);
                i += 1;
                println!("Got message: {}", msg);
            }
        });
        let sender = runtime.spawn(async move {
            for i in 0..10 {
                sender.send(i).await.expect("Failed to send message");
            }
        });

        let result = runtime.block_on(async {
            timeout(Duration::from_secs(5), async {
                // reciever.await.unwrap();
                sender.await.unwrap();
            }).await
        });
        assert!(result.is_ok(), "A potential filled channel is not handled correctly");
    }
}

