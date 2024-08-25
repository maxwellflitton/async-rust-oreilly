
fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration, timeout};

    #[tokio::test]
    async fn test_deadlock_detection() {
        let resource1 = Arc::new(Mutex::new(0));
        let resource2 = Arc::new(Mutex::new(0));

        let resource1_clone = Arc::clone(&resource1);
        let resource2_clone = Arc::clone(&resource2);

        let handle1 = tokio::spawn(async move {
            let _lock1 = resource1.lock().await;
            sleep(Duration::from_millis(100)).await;
            let _lock2 = resource2.lock().await;
        });

        let handle2 = tokio::spawn(async move {
            let _lock2 = resource2_clone.lock().await;
            sleep(Duration::from_millis(100)).await;
            let _lock1 = resource1_clone.lock().await;
        });

        let result = timeout(Duration::from_secs(5), async {
            let _ = handle1.await;
            let _ = handle2.await;
        }).await;
        assert!(result.is_ok(), "A potential deadlock detected!");
    }
}
