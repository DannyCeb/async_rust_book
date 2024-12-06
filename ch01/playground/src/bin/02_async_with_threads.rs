use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Condvar, Mutex,
    },
    thread,
    time::Duration,
};

fn main() {
    let shared_data = Arc::new((Mutex::new(false), Condvar::new()));
    let shared_data_clone = shared_data.clone();
    let STOP = Arc::new(AtomicBool::new(false));
    let STOP_CLONE = STOP.clone();

    let background_thread = thread::spawn(move || {
        let (lock, cvar) = &*shared_data_clone;
        let mut received_value = lock.lock().unwrap();

        while !STOP.load(Relaxed) {
            received_value = cvar.wait(received_value).unwrap();
            println!("Received value: {}", received_value);
        }
    });

    let updater_thread = thread::spawn(move || {
        let (lock, cvar) = &*shared_data;
        let values = [false, true, true, false];

        for l in 0..4 {
            let update_value = values[l];
            println!("updating value to: {}...", update_value);
            *lock.lock().unwrap() = update_value;
            cvar.notify_one();
            thread::sleep(Duration::from_secs(4));
        }
        STOP_CLONE.store(true, Relaxed);
        println!("STOP has been updated");
        cvar.notify_one();
    });

    updater_thread.join().unwrap();
    background_thread.join().unwrap();
}
