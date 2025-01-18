use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::Mutex;

async fn actor_replacement(state: Arc<Mutex<i64>>, value: i64) -> i64 {
    let mut state = state.lock().await;
    *state += value;
    *state
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    let now = tokio::time::Instant::now();
    for i in 0..100_000_000 {
        let state_ref = state.clone();
        let future = async move {
            let handle = tokio::spawn(async move { actor_replacement(state_ref, i).await });
            let _ = handle.await;
        };

        handles.push(tokio::spawn(future));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Elapsed: {:?}", now.elapsed());
}
