use std::{
    io::{self, Cursor, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
    },
    thread,
};

use async_runtime::{executor::Executor, sleep::Sleep};
use data_layer::data::Data;

static FLAGS: [AtomicBool; 3] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

fn main() {
    println!("Hello, world!");
}
