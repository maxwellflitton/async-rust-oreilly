


#[tokio::main]
async fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {

    use tokio::sync::Mutex;
    use tokio::time::{sleep, Duration};
    use tokio_test::{task::spawn, assert_pending};
    use std::sync::Arc;
    use std::task::Poll;

    async fn async_mutex_locker(mutex: Arc<Mutex<i32>>) -> () {
        let mut lock = mutex.lock().await;
        *lock += 1;
        sleep(Duration::from_millis(1)).await;
    }

    #[tokio::test]
    async fn test_monitor_file_metadata() {
        let mutex = Arc::new(Mutex::new(0));
        let mutex_clone1 = mutex.clone();
        let mutex_clone2 = mutex.clone();

        let mut future1 = spawn(async_mutex_locker(mutex_clone1));
        let mut future2 = spawn(async_mutex_locker(mutex_clone2));

        assert_pending!(future1.poll());
        assert_pending!(future2.poll());

        for _ in 0..10 {
            assert_pending!(future2.poll());
            sleep(Duration::from_millis(1)).await;
        }

        assert_eq!(future1.poll(), Poll::Ready(()));
        sleep(Duration::from_millis(3)).await;
        assert_pending!(future2.poll());

        drop(future1);
        sleep(Duration::from_millis(1)).await;
        assert_eq!(future2.poll(), Poll::Ready(()));

        let lock = mutex.lock().await;
        assert_eq!(*lock, 2);
    }
}

