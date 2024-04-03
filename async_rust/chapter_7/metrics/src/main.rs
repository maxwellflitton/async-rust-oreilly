use tokio_util::task::LocalPoolHandle;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use tokio::signal::unix::{signal, SignalKind};


thread_local! {
    pub static COUNTER: UnsafeCell<HashMap<u32, u32>> = UnsafeCell::new(HashMap::new());
}


async fn something(number: u32) {
    tokio::time::sleep(std::time::Duration::from_secs(number as u64)).await;
    COUNTER.with(|counter| {
        let counter = unsafe { &mut *counter.get() };

        match counter.get_mut(&number) {
            Some(count) => {
                let placeholder = *count + 1;
                *count = placeholder;
            },
            None => {
                counter.insert(number, 1);
            }
        }
    });
}


static RUNTIME: Lazy<LocalPoolHandle> = Lazy::new(|| {
    LocalPoolHandle::new(4)
});


fn extract_data_from_thread() -> HashMap<u32, u32> {
    let mut extracted_counter: HashMap<u32, u32> = HashMap::new();
    COUNTER.with(|counter| {
        let counter = unsafe { &mut *counter.get() };
        extracted_counter = counter.clone();
    });
    return extracted_counter
}


async fn get_complete_count() -> HashMap<u32, u32> {
    let mut complete_counter = HashMap::new();
    let mut extracted_counters = Vec::new();

    for i in 0..4 {
        extracted_counters.push(RUNTIME.spawn_pinned_by_idx(|| async move {
            extract_data_from_thread()
        }, i));
    }
    for counter_future in extracted_counters {
        let extracted_counter = counter_future.await.unwrap_or_default();
        for (key, count) in extracted_counter {
            *complete_counter.entry(key).or_insert(0) += count;
        }
    }
    return complete_counter
}



#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _handle = tokio::spawn( async {
        let sequence = [1, 2, 3, 4, 5];
        let repeated_sequence: Vec<_> = sequence.iter().cycle().take(500000).cloned().collect();

        let mut futures = Vec::new();
        for number in repeated_sequence {
            futures.push(RUNTIME.spawn_pinned(move || async move {
                something(number).await;
                something(number).await
            }));
        }
        for i in futures {
            let _ = i.await.unwrap();
        }
        println!("All futures completed");
    });
    let pid = std::process::id();
    println!("The PID of this process is: {}", pid);
    let mut stream = signal(SignalKind::hangup()).unwrap();
    stream.recv().await;
    let complete_counter = get_complete_count().await;
    println!("Complete counter: {:?}", complete_counter);
}
