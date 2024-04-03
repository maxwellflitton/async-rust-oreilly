use std::{future::Future, panic::catch_unwind, thread};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use flume::{Sender, Receiver};

use async_task::{Runnable, Task};
use futures_lite::future;
use once_cell::sync::Lazy;


fn spawn_task<F, T>(future: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static + FutureOrderLabel,
    T: Send + 'static,
{

    static HIGH_CHANNEL: Lazy<(Sender<Runnable>, Receiver<Runnable>)> = Lazy::new(|| 
        {flume::unbounded::<Runnable>()}
    );
    static LOW_CHANNEL: Lazy<(Sender<Runnable>, Receiver<Runnable>)> = Lazy::new(|| 
        {flume::unbounded::<Runnable>()}
    );    

    static HIGH_QUEUE: Lazy<flume::Sender<Runnable>> = Lazy::new(|| {
        for _ in 0..2 {
            let high_receiver = HIGH_CHANNEL.1.clone();
            let low_receiver = LOW_CHANNEL.1.clone();
            thread::spawn(move || {
                loop {
                    match high_receiver.try_recv() {
                        Ok(runnable) => {
                            let _ = catch_unwind(|| runnable.run());
                        },
                        Err(_) => {
                            match low_receiver.try_recv() {
                                Ok(runnable) => {
                                    let _ = catch_unwind(|| runnable.run());
                                },
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    };
                }
            });
        }
        HIGH_CHANNEL.0.clone()
    });
    static LOW_QUEUE: Lazy<flume::Sender<Runnable>> = Lazy::new(|| {
        for _ in 0..2 {
            let high_receiver = HIGH_CHANNEL.1.clone();
            let low_receiver = LOW_CHANNEL.1.clone();
            thread::spawn(move || {
                loop {
                    match low_receiver.try_recv() {
                        Ok(runnable) => {
                            let _ = catch_unwind(|| runnable.run());
                        },
                        Err(_) => {
                            match high_receiver.try_recv() {
                                Ok(runnable) => {
                                    let _ = catch_unwind(|| runnable.run());
                                },
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    };
                }
            });
        }
        LOW_CHANNELs.0.clone()
    });
    
    let schedule_high = |runnable| HIGH_QUEUE.send(runnable).unwrap();
    let schedule_low = |runnable| LOW_QUEUE.send(runnable).unwrap();

    let schedule = match future.get_order() {
        FutureType::High => schedule_high,
        FutureType::Low => schedule_low
    };
    let (runnable, task) = async_task::spawn(future, schedule);
    runnable.schedule();
    return task
}


struct CounterFuture {
    count: u32,
    order: FutureType
}

impl Future for CounterFuture {
    type Output = u32;


    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
        self.count += 1;
        println!("polling with result: {}", self.count);
        std::thread::sleep(Duration::from_secs(1));


        if self.count < 3 {
            cx.waker().wake_by_ref();
        Poll::Pending
        } else {
            Poll::Ready(self.count)
        }
    }
}

impl FutureOrderLabel for CounterFuture {
    fn get_order(&self) -> FutureType {
        self.order
    }
}


async fn async_fn() {
    std::thread::sleep(Duration::from_secs(1));
    println!("async fn");
}


#[derive(Debug, Clone, Copy)]
enum FutureType {
    High,
    Low
}

trait FutureOrderLabel: Future {
    fn get_order(&self) -> FutureType;
}




fn main() {
    let one = CounterFuture { count: 0, order: FutureType::High};
    let two = CounterFuture { count: 0, order: FutureType::Low };


    let t_one = spawn_task(one);
    let t_two = spawn_task(two);
    // let t_three = spawn_task(async {
    //     async_fn().await;
    //     async_fn().await;
    //     async_fn().await;
    //     async_fn().await;
    // });
    std::thread::sleep(Duration::from_secs(5));
    println!("before the block");
    future::block_on(t_one);
    future::block_on(t_two);
    // future::block_on(t_three);
}
