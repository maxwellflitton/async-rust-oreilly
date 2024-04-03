use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};


// Define the possible states of the state machine
enum State {
    On,
    Off,
}

// Define the possible events that can occur
enum Event {
    SwitchOn,
    SwitchOff,
}

// Implement the state machine logic
impl State {
    // This async function transitions the state machine based on the current state and an event
    async fn transition(self, event: Event) -> Self {
        match (&self, event) {
            (State::On, Event::SwitchOff) => {
                println!("Transitioning to the Off state");
                State::Off
            },
            (State::Off, Event::SwitchOn) => {
                println!("Transitioning to the On state");
                State::On
            },
            _ => {
                println!("No transition possible, staying in the current state");
                self
            },
        }
    }
}


struct StateFuture<F: Future, X: Future> {
    pub state: State,
    pub on_future: F,
    pub off_future: X,
}


impl<F: Future, X: Future> Future for StateFuture<F, X> {
    type Output = State;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            State::On => {
                let inner = unsafe { self.map_unchecked_mut(|s| &mut s.on_future) };
                let _ = inner.poll(cx);
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            State::Off => {
                let inner = unsafe { self.map_unchecked_mut(|s| &mut s.off_future) };
                let _ = inner.poll(cx);
                cx.waker().wake_by_ref();
                Poll::Pending
            },
        }
    }
}


struct OnFuture;

impl Future for OnFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("On future");
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
struct OffFuture;

impl Future for OffFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Off future");
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}


#[tokio::main]
async fn main() {
    let mut state = State::On;

    state = state.transition(Event::SwitchOff).await;
    state = state.transition(Event::SwitchOn).await;
    state = state.transition(Event::SwitchOn).await;

    match state {
        State::On => println!("State machine is in the On state"),
        _ => println!("State machine is not in the expected state"),
    }

    let state_future = StateFuture {
        state: State::On,
        on_future: OnFuture{},
        off_future: OffFuture{},
    };
    let _ = state_future.await;
}
