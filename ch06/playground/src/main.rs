//use device_query::{DeviceEvents, DeviceState};
use std::cell::{LazyCell, RefCell};
use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

const TEMP: LazyCell<Rc<RefCell<i16>>> = LazyCell::new(|| Rc::new(RefCell::new(1010)));
const DESIRED_TEMP: LazyCell<Rc<RefCell<i16>>> = LazyCell::new(|| Rc::new(RefCell::new(1200)));
const HEAT_ON: LazyCell<Rc<RefCell<bool>>> = LazyCell::new(|| Rc::new(RefCell::new(false)));

//const INPUT: LazyCell<Rc<RefCell<String>>> = LazyCell::new(|| Rc::new(RefCell::new(String::new())));
//const DEVICE_STATE: LazyCell<Rc<DeviceState>> = LazyCell::new(|| Rc::new(DeviceState::new()));

pub fn render(temp: i16, desired_temp: i16, heat_on: bool, input: String) {
    clearscreen::clear().unwrap();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    println!(
        "Temperature: {}\nDesired Temp: {}\nHeater On: {}",
        temp as f32 / 100.0,
        desired_temp as f32 / 100.0,
        heat_on
    );
    print!("Input: {}", input);
    handle.flush().unwrap();
}

pub struct DisplayFuture {
    pub temp_snapshot: i16,
}

impl DisplayFuture {
    pub fn new() -> Self {
        DisplayFuture {
            temp_snapshot: *TEMP.clone().as_ref().borrow(),
        }
    }
}

impl Future for DisplayFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_snapshot = *TEMP.clone().as_ref().borrow();
        let desired_temp = *DESIRED_TEMP.clone().as_ref().borrow();
        let heat_on = *HEAT_ON.clone().as_ref().borrow();

        if current_snapshot == self.temp_snapshot {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            if current_snapshot < desired_temp && heat_on == false {
                *HEAT_ON.clone().as_ref().borrow_mut() = true;
            } else if current_snapshot > desired_temp && heat_on == true {
                *HEAT_ON.clone().as_ref().borrow_mut() = false;
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
        if !*HEAT_ON.clone().as_ref().borrow() {
            self.time_snapshot = Instant::now();
            cx.waker().wake_by_ref();
            Poll::Pending
        // Check if the time since the last increment is less than 3 seconds
        } else if Instant::now().duration_since(self.time_snapshot) < Duration::from_secs(3) {
            cx.waker().wake_by_ref();
            Poll::Pending
        // Increment the temperature by 3
        } else {
            let ref_mut = TEMP.clone();
            let mut value_mut = ref_mut.as_ref().borrow_mut();
            *value_mut += 3;
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
            let ref_mut = TEMP.clone();
            let mut value_mut = ref_mut.as_ref().borrow_mut();
            *value_mut -= 1;
            self.time_snapshot = Instant::now();
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    /*let _guard = DEVICE_STATE.on_key_down(|key| {
        {
            let input = INPUT.clone();

            input.as_ref().borrow_mut().push_str(&key.to_string());
        }
        render(
            *TEMP.clone().as_ref().borrow(),
            *DESIRED_TEMP.clone().as_ref().borrow(),
            *HEAT_ON.clone().as_ref().borrow(),
            INPUT.clone().as_ref().borrow().clone(),
        );
    });*/

    let display = tokio::spawn(async { DisplayFuture::new().await });
    let heat_loss = tokio::spawn(async { HeatLossFuture::new().await });
    let heater = tokio::spawn(async { HeaterFuture::new().await });

    display.await.unwrap();
    heat_loss.await.unwrap();
    heater.await.unwrap();
}
