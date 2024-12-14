use std::pin::Pin;
use std::sync::LazyLock;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::{future::Future, panic::catch_unwind, thread};

use async_task::{Runnable, Task};
use flume::{Receiver, Sender};
use futures_lite::future;

#[derive(Debug, Clone, Copy)]
enum FutureType {
    High,
    Low,
}

macro_rules! try_join {
    ($($future:expr), *) => {
        {
            let mut results = Vec::new();
            $(
                results.push( catch_unwind(|| future::block_on($future)));
            )*
            results
        }
    };
}

macro_rules! join {
    ($($future:expr), *) => {
        {
            let mut results = Vec::new();
            $(
                results.push(future::block_on($future));
            )*
            results
        }
    };
}

macro_rules! spawn_task {
    ($future:expr) => {
        spawn_task!($future, FutureType::Low)
    };
    ($future:expr, $order:expr) => {
        spawn_task($future, $order)
    };
}
fn spawn_task<F, T>(future: F, order: FutureType) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    static HIGH_CHANNEL: LazyLock<(Sender<Runnable>, Receiver<Runnable>)> =
        LazyLock::new(|| flume::unbounded::<Runnable>());

    static LOW_CHANNEL: LazyLock<(Sender<Runnable>, Receiver<Runnable>)> =
        LazyLock::new(|| flume::unbounded::<Runnable>());

    let queue = match order {
        FutureType::High => {
            static HIGH_QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
                let high_num = std::env::var("HIGH_NUM").unwrap().parse::<usize>().unwrap();
                for _ in 0..high_num {
                    let high_receiver = HIGH_CHANNEL.1.clone();
                    let low_receiver = LOW_CHANNEL.1.clone();
                    thread::spawn(move || loop {
                        match high_receiver.try_recv() {
                            Ok(runnable) => {
                                let _ = catch_unwind(|| runnable.run());
                            }
                            Err(_) => match low_receiver.try_recv() {
                                Ok(runnable) => {
                                    println!("running low queue stuff on hight one");
                                    let _ = catch_unwind(|| runnable.run());
                                }
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(100));
                                }
                            },
                        }
                    });
                }
                HIGH_CHANNEL.0.clone()
            });

            &HIGH_QUEUE
        }
        FutureType::Low => {
            static LOW_QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
                let low_num = std::env::var("LOW_NUM").unwrap().parse::<usize>().unwrap();
                for _ in 0..low_num {
                    let high_receiver = HIGH_CHANNEL.1.clone();
                    let low_receiver = LOW_CHANNEL.1.clone();
                    thread::spawn(move || loop {
                        match low_receiver.try_recv() {
                            Ok(runnable) => {
                                let _ = catch_unwind(|| runnable.run());
                            }
                            Err(_) => match high_receiver.try_recv() {
                                Ok(runnable) => {
                                    let _ = catch_unwind(|| runnable.run());
                                }
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(100));
                                }
                            },
                        }
                    });
                }
                LOW_CHANNEL.0.clone()
            });

            &LOW_QUEUE
        }
    };

    let schedule = |runnable| queue.send(runnable).unwrap();

    let (runnable, task) = async_task::spawn(future, schedule);

    runnable.schedule();

    task
}

struct CounterFuture {
    count: u32,
}

impl Future for CounterFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.count += 1;
        println!("polling with result: {}", self.count);

        std::thread::sleep(Duration::from_secs(1));

        if self.count < 3 {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(self.count)
        }
    }
}

async fn async_fn() {
    std::thread::sleep(Duration::from_secs(1));
    println!("Async fn");
}

struct AsyncSleep {
    start_time: Instant,
    duration: Duration,
}

impl AsyncSleep {
    fn new(duration: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
        }
    }
}

impl Future for AsyncSleep {
    type Output = bool;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let elapsed_time = self.start_time.elapsed();

        if elapsed_time >= self.duration {
            Poll::Ready(true)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct Runtime {
    high_num: usize,
    low_num: usize,
}

impl Runtime {
    pub fn new() -> Self {
        let num_cores = std::thread::available_parallelism().unwrap().get();
        Self {
            high_num: num_cores - 2,
            low_num: 1,
        }
    }

    pub fn with_high_num(mut self, num: usize) -> Self {
        self.high_num = num;
        self
    }

    pub fn with_low_num(mut self, num: usize) -> Self {
        self.low_num = num;
        self
    }

    pub fn run(&self) {
        std::env::set_var("HIGH_NUM", self.high_num.to_string());
        std::env::set_var("LOW_NUM", self.low_num.to_string());

        let high = spawn_task!(async {}, FutureType::High);
        let low = spawn_task!(async {}, FutureType::Low);

        join!(high, low);
    }
}

struct BackgroundProcess;

impl Future for BackgroundProcess {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Background process firing");
        std::thread::sleep(Duration::from_secs(1));
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

fn main() {
    Runtime::new().with_low_num(3).with_high_num(8).run();
    let _ = spawn_task!(BackgroundProcess {}).detach();

    let one = CounterFuture { count: 0 };
    let two = CounterFuture { count: 0 };
    let four = CounterFuture { count: 0 };
    let t_one = spawn_task!(one, FutureType::High);
    let t_two = spawn_task!(two);
    let t_four = spawn_task!(four);
    let t_three = spawn_task!(async {
        async_fn().await;
        async_fn().await;
        async_fn().await;
        async_fn().await;
    });

    let _a = join!(t_one, t_two, t_four);
    let _a = join!(t_three);
}
