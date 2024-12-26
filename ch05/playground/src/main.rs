#![feature(coroutines, coroutine_trait)]
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub struct MutexCoRoutine {
    pub handle: Arc<Mutex<u8>>,
    pub threshold: u8,
}

impl Coroutine<()> for MutexCoRoutine {
    type Yield = ();
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, _arg: ()) -> CoroutineState<Self::Yield, Self::Return> {
        match self.handle.clone().try_lock() {
            Ok(mut handle) => {
                *handle += 1;
                self.threshold -= 1;

                if self.threshold == 0 {
                    CoroutineState::Complete(())
                } else {
                    CoroutineState::Yielded(())
                }
            }
            Err(_) => CoroutineState::Yielded(()),
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {

    use super::*;
    use std::future::Future;
    use std::task::{Context, Poll};
    use std::time::Duration;

    // sync testing interface

    fn check_yield(coroutine: &mut MutexCoRoutine) -> bool {
        match Pin::new(coroutine).resume(()) {
            CoroutineState::Yielded(_) => true,
            _ => false,
        }
    }

    impl Future for MutexCoRoutine {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.resume(()) {
                CoroutineState::Yielded(_) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                CoroutineState::Complete(_) => Poll::Ready(()),
            }
        }
    }

    #[test]
    fn basic_test() {
        let handle = Arc::new(Mutex::new(0));

        let mut first_coroutine = MutexCoRoutine {
            handle: handle.clone(),
            threshold: 2,
        };

        let mut second_coroutine = MutexCoRoutine {
            handle: handle.clone(),
            threshold: 2,
        };

        let lock = handle.lock().unwrap();

        for _ in 0..2 {
            assert!(check_yield(&mut first_coroutine));
            assert!(check_yield(&mut second_coroutine));
        }

        assert_eq!(*lock, 0);

        std::mem::drop(lock);

        assert_eq!(check_yield(&mut first_coroutine), true);
        assert_eq!(*handle.lock().unwrap(), 1);
        assert_eq!(check_yield(&mut second_coroutine), true);
        assert_eq!(*handle.lock().unwrap(), 2);
        assert_eq!(check_yield(&mut first_coroutine), false);
        assert_eq!(*handle.lock().unwrap(), 3);
        assert_eq!(check_yield(&mut second_coroutine), false);
        assert_eq!(*handle.lock().unwrap(), 4);
    }

    #[tokio::test]
    async fn async_test() {
        let handle = Arc::new(Mutex::new(0));

        let first_coroutine = MutexCoRoutine {
            handle: handle.clone(),
            threshold: 2,
        };

        let second_coroutine = MutexCoRoutine {
            handle: handle.clone(),
            threshold: 2,
        };
        let handle_one = tokio::spawn(async move { first_coroutine.await });
        let handle_two = tokio::spawn(async move { second_coroutine.await });

        let _ = handle_one.await;
        let _ = handle_two.await;

        assert_eq!(*handle.lock().unwrap(), 4);
    }
}
