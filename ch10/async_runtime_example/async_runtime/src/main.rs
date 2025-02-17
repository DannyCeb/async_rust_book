use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use async_runtime::executor::Executor;

pub struct CountingFuture {
    pub count: i32,
}

impl Future for CountingFuture {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.count < 4 {
            self.count += 1;
            cx.waker().wake_by_ref();
            println!("countingFuture is not done yet! {}", self.count);
            Poll::Pending
        } else {
            println!("CountingFuture is done!");
            Poll::Ready(self.count)
        }
    }
}

fn main() {
    let counter = CountingFuture { count: 0 };
    let counter_two = CountingFuture { count: 0 };

    let mut executor = Executor::new();

    let handle = executor.spawn(counter);
    let _handle_two = executor.spawn(counter_two);

    std::thread::spawn(move || loop {
        executor.poll();
    });

    let result = handle.recv().unwrap();
    println!("result: {}", result);
}
