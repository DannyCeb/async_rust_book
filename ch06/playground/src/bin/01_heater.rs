use core::sync::atomic::Ordering;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicI16};
use std::sync::Arc;
use std::sync::LazyLock;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

static TEMP: LazyLock<Arc<AtomicI16>> = LazyLock::new(|| Arc::new(AtomicI16::new(1010)));
static DESIRED_TEMP: LazyLock<Arc<AtomicI16>> = LazyLock::new(|| Arc::new(AtomicI16::new(1200)));
static HEAT_ON: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

pub struct DisplayFuture {
    pub temp_snapshot: i16,
}

impl DisplayFuture {
    pub fn new() -> Self {
        DisplayFuture {
            temp_snapshot: TEMP.load(Ordering::SeqCst),
        }
    }
}

impl Future for DisplayFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_snapshot = TEMP.load(Ordering::SeqCst);
        let desired_temp = DESIRED_TEMP.load(Ordering::SeqCst);
        let heat_on = HEAT_ON.load(Ordering::SeqCst);

        if current_snapshot == self.temp_snapshot {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            if current_snapshot < desired_temp && heat_on == false {
                HEAT_ON.store(true, Ordering::SeqCst);
            } else if current_snapshot > desired_temp && heat_on == true {
                HEAT_ON.store(false, Ordering::SeqCst);
            }
            clearscreen::clear().unwrap();
            println!(
                "Temperature: {}\n Desired temp: {}\n Heater On: {}",
                current_snapshot as f32 / 100.0,
                desired_temp as f32 / 100.0,
                heat_on
            );

            self.temp_snapshot = current_snapshot;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub struct HeaterFuture {
    pub time_snapshot: Instant,
}

impl HeaterFuture {
    pub fn new() -> Self {
        HeaterFuture {
            time_snapshot: Instant::now(),
        }
    }
}

impl Future for HeaterFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the heater is on
        if !HEAT_ON.load(Ordering::SeqCst) {
            self.time_snapshot = Instant::now();
            cx.waker().wake_by_ref();
            Poll::Pending
        // Check if the time since the last increment is less than 3 seconds
        } else if Instant::now().duration_since(self.time_snapshot) < Duration::from_secs(3) {
            cx.waker().wake_by_ref();
            Poll::Pending
        // Increment the temperature by 3
        } else {
            TEMP.fetch_add(3, Ordering::SeqCst);
            self.time_snapshot = Instant::now();
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct HeatLossFuture {
    pub time_snapshot: Instant,
}

impl HeatLossFuture {
    pub fn new() -> Self {
        HeatLossFuture {
            time_snapshot: Instant::now(),
        }
    }
}

impl Future for HeatLossFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now().duration_since(self.time_snapshot) > Duration::from_secs(3) {
            TEMP.fetch_sub(1, Ordering::SeqCst);
            self.time_snapshot = Instant::now();
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

#[tokio::main]
async fn main() {
    let display = tokio::spawn(async { DisplayFuture::new().await });
    let heat_loss = tokio::spawn(async { HeatLossFuture::new().await });
    let heater = tokio::spawn(async { HeaterFuture::new().await });

    let _ = display.await;
    /*let _ = heat_loss.await;
    let _ = heater.await;*/
}
