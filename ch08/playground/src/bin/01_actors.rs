use tokio::sync::{
    mpsc::channel,
    mpsc::{Receiver, Sender},
    oneshot,
};

struct Message {
    value: u64,
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
    responder: oneshot::Sender<i32>,
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
    let (tx, rx) = channel::<RespMessage>(100);

    let _resp_actor_handle = tokio::spawn(resp_actor(rx));

    for i in 0..10 {
        let (resp_tx, resp_rx) = oneshot::channel::<i32>();
        let msg = RespMessage {
            value: i,
            responder: resp_tx,
        };

        tx.send(msg).await.unwrap();
        println!("Response: {}", resp_rx.await.unwrap());
    }
}
