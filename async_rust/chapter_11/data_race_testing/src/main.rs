


// Pin<Box<dyn Future<Output = T> + Send + 'static>>



fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {

    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::runtime::Builder;
    use tokio::time::{sleep, Duration};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    async fn unsafe_add() {
        let value = COUNTER.load(Ordering::SeqCst);
        sleep(Duration::from_secs(1)).await;
        COUNTER.store(value + 1, Ordering::SeqCst);
    }

    #[test]
    fn test_data_race() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut handles = vec![];
        let total = 100000;

        for _ in 0..total {
            let handle = runtime.spawn(unsafe_add());
            handles.push(handle);
        }

        for handle in handles {
            runtime.block_on(handle).unwrap();
        }

        assert_eq!(COUNTER.load(Ordering::SeqCst), total, "Data race occurred!");
    }
}

