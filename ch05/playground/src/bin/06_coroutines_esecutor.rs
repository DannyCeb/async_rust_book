#![feature(coroutines, coroutine_trait)]

use std::{
    collections::VecDeque,
    future::Future,
    ops::{Coroutine, CoroutineState},
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

struct SleepCoroutine {
    pub start: Instant,
    pub duration: Duration,
}

impl SleepCoroutine {
    pub fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }
}

impl Coroutine<()> for SleepCoroutine {
    type Yield = ();
    type Return = ();
    fn resume(self: Pin<&mut Self>, _: ()) -> CoroutineState<Self::Yield, Self::Return> {
        if Instant::now() - self.start >= self.duration {
            CoroutineState::Complete(())
        } else {
            CoroutineState::Yielded(())
        }
    }
}

struct Executor {
    coroutines: VecDeque<Pin<Box<dyn Coroutine<(), Yield = (), Return = ()>>>>,
}

impl Executor {
    fn new() -> Self {
        Self {
            coroutines: VecDeque::new(),
        }
    }

    fn add(&mut self, coroutine: Pin<Box<dyn Coroutine<(), Yield = (), Return = ()>>>) {
        self.coroutines.push_back(coroutine);
    }

    fn poll(&mut self) {
        println!("Polling {} coroutines", self.coroutines.len());

        let mut coroutine = self.coroutines.pop_front().unwrap();

        match coroutine.as_mut().resume(()) {
            CoroutineState::Yielded(_) => {
                self.coroutines.push_back(coroutine);
            }
            CoroutineState::Complete(_) => {}
        }
    }
}

impl Future for SleepCoroutine {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.resume(()) {
            CoroutineState::Complete(_) => Poll::Ready(()),
            CoroutineState::Yielded(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

fn main() {
    let mut executor = Executor::new();

    for _ in 0..3 {
        let coroutine = SleepCoroutine::new(Duration::from_secs(1));
        executor.add(Box::pin(coroutine));
    }

    let start = Instant::now();

    while !executor.coroutines.is_empty() {
        executor.poll();
    }

    println!("Elapsed time: {:?}", start.elapsed());
}
