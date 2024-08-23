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
use rand::Rng;
use std::time::Duration;

struct RandCoRoutine {
    pub value: u8,
    pub live: bool,
}
impl RandCoRoutine {
    fn new() -> Self {
        let mut coroutine = Self {
            value: 0,
            live: true,
        };
        coroutine.generate();
        coroutine
    }
    fn generate(&mut self) {
        let mut rng = rand::thread_rng();
        self.value = rng.gen_range(0..=10);
    }
}

impl Coroutine<()> for RandCoRoutine {
    type Yield = u8;
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, _: ()) 
        -> CoroutineState<Self::Yield, Self::Return> {
        self.generate();
        CoroutineState::Yielded(self.value)
    }
}


fn main() {
    let mut coroutines = Vec::new();
    for _ in 0..10 {
        coroutines.push(RandCoRoutine::new());
    }
    let mut total: u32 = 0;

    loop {
        let mut all_dead = true;
        for mut coroutine in coroutines.iter_mut() {
            if coroutine.live {
                all_dead = false;
                match Pin::new(&mut coroutine).resume(()) {
                    CoroutineState::Yielded(result) => {
                        total += result as u32;
                    },
                    CoroutineState::Complete(_) => {
                        panic!("Coroutine should not complete");
                    },
                }
                if coroutine.value < 9 {
                    coroutine.live = false;
                }
            }
        }
        if all_dead {
            break
        }
    }
    println!("Total: {}", total);

    let (sender, reciever) = std::sync::mpsc::channel::<RandCoRoutine>();
    let thread = std::thread::spawn(move || {
        loop {
            let mut coroutine = reciever.recv().unwrap();
            match Pin::new(&mut coroutine).resume(()) {
                CoroutineState::Yielded(result) => {
                    println!("Coroutine yielded: {}", result);
                },
                CoroutineState::Complete(_) => {
                    panic!("Coroutine should not complete");
                },
            }
        }
    });
    std::thread::sleep(Duration::from_secs(1));
    sender.send(RandCoRoutine::new()).unwrap();
    sender.send(RandCoRoutine::new()).unwrap();
    std::thread::sleep(Duration::from_secs(1));
}
