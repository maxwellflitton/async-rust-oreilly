#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(associated_type_bounds)]
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;


struct SleepCoroutine {}


impl Coroutine for SleepCoroutine {
    type Yield = ();
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, arg: i32) 
        -> CoroutineState<Self::Yield, Self::Return> {
        CoroutineState::Yielded(())
    }
}

fn main() {

    println!("Main task is done.");
}