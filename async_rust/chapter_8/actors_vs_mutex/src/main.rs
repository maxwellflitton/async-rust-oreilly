use tokio::sync::{
    mpsc::channel,
    mpsc::Receiver,
    oneshot,
    Mutex,
};
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;


use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver as CrossbeamReceiver;


struct RespMessage {
    value: i32,
    responder: oneshot::Sender<i32>
}

async fn resp_actor(mut rx: Receiver<RespMessage>) {
    let mut state = 0;

    while let Some(msg) = rx.recv().await {
        state += msg.value;

        if msg.responder.send(state).is_err() {
            eprintln!("Failed to send response");
        }
    }
}

async fn greedy_resp_actor(mut rx: Receiver<RespMessage>) {
    let mut state = 0;

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                state += msg.value;
                if msg.responder.send(state).is_err() {
                    eprintln!("Failed to send response");
                }
            },
            Err(e) => {
                if e == TryRecvError::Empty {
                    // No message, do a brief pause to yield control
                    tokio::task::yield_now().await;
                } else {
                    // Receiver has been closed
                    break;
                }
            }
        }
    }
}



async fn offload_resp_actor(mut rx: Receiver<RespMessage>) {
    let (off_tx, off_rx) = unbounded::<RespMessage>();

    let mut handles = Vec::new();

    for _ in 0..4 {
        let off_rx_ref = off_rx.clone();
        let future = async move {
            offload_reciever(off_rx_ref).await;
        };
        handles.push(tokio::spawn(future));
    }

    let mut state = 0;

    while let Some(msg) = rx.recv().await {
        state += msg.value;

        let message = RespMessage {
            value: state,
            responder: msg.responder
        };

        if off_tx.send(message).is_err() {
            eprintln!("Failed to send response");
        }
    }
}


async fn offload_reciever(rx: CrossbeamReceiver<RespMessage>) {
    while let Ok(msg) = rx.recv() {
        if msg.responder.send(msg.value).is_err() {
            eprintln!("Failed to send response");
        }
    }
}


async fn batch_resp_actor(mut rx: Receiver<RespMessage>) {
    let mut state = 0;

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                // Process the message immediately
                state += msg.value;
                if msg.responder.send(state).is_err() {
                    eprintln!("Failed to send response");
                }
            },
            Err(TryRecvError::Empty) => {
                // Channel is empty, now await for next message
                if let Some(msg) = rx.recv().await {
                    state += msg.value;
                    if msg.responder.send(state).is_err() {
                        eprintln!("Failed to send response");
                    }
                } else {
                    // Channel has been closed
                    break;
                }
            },
            Err(TryRecvError::Disconnected) => {
                // Channel has been closed
                break;
            }
        }
    }
}


async fn actor_replacement(state: Arc<Mutex<i32>>, value: i32) -> i32 {
    // let update_handle = tokio::spawn(async move {
    //     let mut state = state.lock().await;
    //     *state += value;
    //     return *state
    // });
    let mut state = state.lock().await;
    *state += value;
    return *state
    // update_handle.await.unwrap()
}


#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<RespMessage>(100000000);
    let _resp_actor_handle = tokio::spawn(async {
        resp_actor(rx).await;
    });

    let mut handles = Vec::new();

    let now = tokio::time::Instant::now();
    for i in 0..100000000 {

        let tx_ref = tx.clone();

        let future = async move {
            let (resp_tx, resp_rx) = oneshot::channel::<i32>();
            let msg = RespMessage {
                value: i,
                responder: resp_tx
            };
            tx_ref.send(msg).await.unwrap();
            // do something else
            let _ = resp_rx.await.unwrap();
        };
        handles.push(tokio::spawn(future));
    }
    for handle in handles {
        let _ = handle.await.unwrap();
    }
    println!("Elapsed: {:?}", now.elapsed());


    // mutex


    let state = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    let now = tokio::time::Instant::now();

    for i in 0..100000000 {

        let state_ref = state.clone();

        let future = async move {
            let handle = tokio::spawn(async move {
                actor_replacement(state_ref, i).await
            });
            let _ = handle.await.unwrap();
        };
        handles.push(tokio::spawn(future));
    }
    for handle in handles {
        let _ = handle.await.unwrap();
    }
    println!("Elapsed: {:?}", now.elapsed());
}
