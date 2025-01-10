use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::task::LocalPoolHandle;

thread_local! {
    pub static COUNTER: UnsafeCell<HashMap<u32,u32>> = UnsafeCell::new(HashMap::new());
}

async fn something(number: u32) {
    tokio::time::sleep(Duration::from_secs(number as u64)).await;
    COUNTER.with(|counter| {
        let counter = unsafe { &mut *counter.get() };

        match counter.get_mut(&number) {
            Some(count) => {
                *count += 1;
            }
            None => {
                counter.insert(number, 1);
            }
        }

        println!("Counter: {:?}", counter);
    });
}

async fn print_statement() {
    sleep(Duration::from_secs(1)).await;
    COUNTER.with(|counter| {
        let counter = unsafe { &mut *counter.get() };
        println!("Counter: {:?}", counter);
    });
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pool = LocalPoolHandle::new(1);
    let sequence = [1, 2, 3, 4, 5];

    let repeated_sequence: Vec<_> = sequence.iter().cycle().take(50).cloned().collect();

    println!("{:?}", repeated_sequence);

    let mut futures: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    //let _ = tokio::spawn(async {
    //    loop {
    //        print_statement().await
    //    }
    //});

    for number in repeated_sequence {
        futures.push(pool.spawn_pinned(move || async move {
            something(number).await;
        }));
    }

    for i in futures {
        let _ = i.await.unwrap();
    }

    let _ = pool
        .spawn_pinned(|| async { print_statement().await })
        .await
        .unwrap();
}
