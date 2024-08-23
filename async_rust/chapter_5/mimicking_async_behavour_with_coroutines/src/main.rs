#![feature(coroutines)]
#![feature(coroutine_trait)]
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::VecDeque;
use std::time::Instant;


struct SleepCoroutine {
    pub start: Instant,
    pub duration: std::time::Duration,
}
impl SleepCoroutine {
    fn new(duration: std::time::Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }
}
impl Coroutine<()> for SleepCoroutine {
    type Yield = ();
    type Return = ();

    fn resume(
        self: Pin<&mut Self>, _: ()) 
    -> CoroutineState<Self::Yield, Self::Return> {
        if self.start.elapsed() >= self.duration {
            CoroutineState::Complete(())
        } else {
            CoroutineState::Yielded(())
        }
    }
}

impl Future for SleepCoroutine {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
        match Pin::new(&mut self).resume(()) {
            CoroutineState::Complete(_) => Poll::Ready(()),
            CoroutineState::Yielded(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            },
        }
    }
}


struct Executor {
    coroutines: VecDeque<Pin<Box<
        dyn Coroutine<(), Yield = (), Return = ()>
    >>>,
}

impl Executor {
    fn new() -> Self {
        Self {
            coroutines: VecDeque::new(),
        }
    }
    fn add(&mut self, coroutine: Pin<Box<
        dyn Coroutine<(), Yield = (), Return = ()>>>) 
    {
        self.coroutines.push_back(coroutine);
    }

    fn poll(&mut self) {
        println!("Polling {} coroutines", self.coroutines.len());
        let mut coroutine = self.coroutines.pop_front().unwrap();
        match coroutine.as_mut().resume(()) {
            CoroutineState::Yielded(_) => {
                self.coroutines.push_back(coroutine);
            },
            CoroutineState::Complete(_) => {},
        }
    }
}

fn main() {
    let mut executor = Executor::new();

    for _ in 0..3 {
        let coroutine = SleepCoroutine::new(
            std::time::Duration::from_secs(1)
        );
        executor.add(Box::pin(coroutine));
    }
    let start = Instant::now();
    while !executor.coroutines.is_empty() {
        executor.poll();
    }
    println!("Took {:?}", start.elapsed());
}

// fn main() {
//     let mut sleep_coroutines = VecDeque::new();
//     sleep_coroutines.push_back(
//         SleepCoroutine::new(std::time::Duration::from_secs(1))
//     );
//     sleep_coroutines.push_back(
//         SleepCoroutine::new(std::time::Duration::from_secs(1))
//     );
//     sleep_coroutines.push_back(
//         SleepCoroutine::new(std::time::Duration::from_secs(1))
//     );

//     let mut counter = 0;
//     let start = Instant::now();

//     while counter < sleep_coroutines.len() {
//         let mut coroutine = sleep_coroutines.pop_front().unwrap();
//         match Pin::new(&mut coroutine).resume(()) {
//             CoroutineState::Yielded(_) => {
//                 sleep_coroutines.push_back(coroutine);
//             },
//             CoroutineState::Complete(_) => {
//                 counter += 1;
//             },
//         }
//     }
//     println!("Took {:?}", start.elapsed());
// }
