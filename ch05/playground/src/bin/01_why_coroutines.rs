#![feature(coroutines)]
#![feature(coroutine_trait)]

use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::time::Instant;

use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

struct WriteCoroutine {
    file_handle: File,
}

impl WriteCoroutine {
    fn new(path: &str) -> io::Result<Self> {
        let file_handle = OpenOptions::new().append(true).create(true).open(path)?;
        Ok(Self { file_handle })
    }
}

impl Coroutine<i32> for WriteCoroutine {
    type Yield = ();
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, arg: i32) -> CoroutineState<Self::Yield, Self::Return> {
        writeln!(self.file_handle, "{}", arg).unwrap();
        CoroutineState::Yielded(())
    }
}

fn append_number_to_file(n: i32) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("numbers.txt")?;
    writeln!(file, "{}", n)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let numbers: Vec<i32> = (0..200_000).map(|_| rng.gen()).collect();

    let mut coroutine = WriteCoroutine::new("numbers.txt")?;

    let start = Instant::now();

    for number in numbers {
        Pin::new(&mut coroutine).resume(number);
    }

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);

    Ok(())
}
