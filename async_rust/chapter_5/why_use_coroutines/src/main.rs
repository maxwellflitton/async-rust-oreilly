#![feature(coroutines)]
#![feature(coroutine_trait)]
use std::fs::{OpenOptions, File};
use std::io::{Write, self};
use std::time::Instant;
use rand::Rng;
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

fn append_number_to_file(n: i32) -> io::Result<()> {
    let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("numbers.txt")?;
    writeln!(file, "{}", n)?;
    Ok(())
}


struct WriteCoroutine {
    pub file_handle: File,
}
impl WriteCoroutine {
    fn new() -> Self {
        Self {
            file_handle: OpenOptions::new()
                .create(true)
                .append(true)
                .open("numbers.txt")
                .unwrap(),
        }
    }
}

impl Coroutine<i32> for WriteCoroutine {
    type Yield = ();
    type Return = ();
    fn resume(mut self: Pin<&mut Self>, arg: i32)
        -> CoroutineState<Self::Yield, Self::Return> {
        writeln!(self.file_handle, "{}", arg).unwrap();
        CoroutineState::Yielded(())
    }
}


fn main() {
    let mut rng = rand::thread_rng();
    let numbers: Vec<i32> = (0..200000).map(|_| rng.gen()).collect();
    let start = Instant::now();

    let mut coroutine = WriteCoroutine::new();
    for &number in &numbers {
        Pin::new(&mut coroutine).resume(number);
    }

    // for &number in &numbers {
    //     if let Err(e) = append_number_to_file(number) {
    //         eprintln!("Failed to write to file: {}", e);
    //     }
    // }
    let duration = start.elapsed();
    println!("Time elapsed in file operations is: {:?}", duration);
}
