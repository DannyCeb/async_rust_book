use device_query::{DeviceEvents, DeviceState};
use std::io::{self, Write};
use std::sync::Mutex;

fn perform_operation_with_callback<F>(callback: F)
where
    F: Fn(i32),
{
    let result = 42;
    callback(result)
}

fn main() {
    let my_callback = |result| {
        println!("The result is: {}", result);
    };

    perform_operation_with_callback(my_callback);
}
