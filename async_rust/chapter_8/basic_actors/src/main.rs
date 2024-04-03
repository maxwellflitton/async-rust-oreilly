use tokio::sync::{
    mpsc::channel,
    mpsc::Receiver,
    oneshot,
};


struct Message {
    value: i32
}


async fn basic_actor(mut rx: Receiver<Message>) {

    let mut state = 0;

    while let Some(msg) = rx.recv().await {
        state += msg.value;
        println!("Received: {}", msg.value);
        println!("State: {}", state);
    }
}


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


#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<Message>(10);

    let _actor_handle = tokio::spawn(async {
        basic_actor(rx).await;
    });
    for i in 0..10 {
        let msg = Message { value: i };
        tx.send(msg).await.unwrap();
    }

    let (tx, rx) = channel::<RespMessage>(10);
    let _resp_actor_handle = tokio::spawn(async {
        resp_actor(rx).await;
    });
    for i in 0..10 {
        let (resp_tx, resp_rx) = oneshot::channel::<i32>();
        let msg = RespMessage {
            value: i,
            responder: resp_tx
        };
        tx.send(msg).await.unwrap();
        let _ = resp_rx.await.unwrap();
    }
}
