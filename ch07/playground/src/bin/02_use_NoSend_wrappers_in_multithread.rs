use std::{
    cell::RefCell,
    thread,
    time::{Duration, Instant},
};
use tokio_util::task::LocalPoolHandle;

thread_local! {
    pub static COUNTER: RefCell<u32> = RefCell::new(1);
}

async fn something(number: u32) -> u32 {
    std::thread::sleep(Duration::from_secs(3));
    COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
        println!("Counter: {} for: {}", *counter.borrow(), number)
    });
    number
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();

    let pool = LocalPoolHandle::new(3);

    let one = pool.spawn_pinned(|| async {
        println!("one");
        something(1).await
    });

    let two = pool.spawn_pinned(|| async {
        println!("two");
        something(2).await
    });

    let three = pool.spawn_pinned(|| async {
        println!("three");
        something(3).await
    });

    let result = async { one.await.unwrap() + two.await.unwrap() + three.await.unwrap() };

    println!("Result: {}", result.await);
    println!("took: {}", start.elapsed().as_secs())
}
