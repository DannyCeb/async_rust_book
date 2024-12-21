#![feature(coroutines)]
#![feature(coroutine_trait)]

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader};

use std::io::Write;
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

struct ReadCoroutine {
    lines: io::Lines<BufReader<File>>,
}

impl ReadCoroutine {
    fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let lines = BufReader::new(file).lines();
        Ok(Self { lines })
    }
}

impl Coroutine<()> for ReadCoroutine {
    type Yield = i32;
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, _arg: ()) -> CoroutineState<Self::Yield, Self::Return> {
        match self.lines.next() {
            Some(Ok(line)) => {
                if let Ok(number) = line.parse::<i32>() {
                    CoroutineState::Yielded(number)
                } else {
                    CoroutineState::Complete(())
                }
            }
            _ => CoroutineState::Complete(()),
        }
    }
}

struct WriteCoroutine {
    file_handle: File,
}

impl WriteCoroutine {
    fn new(path: &str) -> io::Result<Self> {
        let file_handle = OpenOptions::new().create(true).append(true).open(path)?;
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

struct CoroutineManager {
    writer: WriteCoroutine,
    reader: ReadCoroutine,
}

impl CoroutineManager {
    fn new(read_path: &str, write_path: &str) -> io::Result<Self> {
        let writer = WriteCoroutine::new(write_path)?;
        let reader = ReadCoroutine::new(read_path)?;
        Ok(Self { writer, reader })
    }

    fn run(&mut self) {
        let mut read_pin = Pin::new(&mut self.reader);
        let mut write_pin = Pin::new(&mut self.writer);

        loop {
            match read_pin.as_mut().resume(()) {
                CoroutineState::Yielded(number) => {
                    write_pin.as_mut().resume(number);
                }
                CoroutineState::Complete(()) => {
                    break;
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut manager = CoroutineManager::new("numbers.txt", "output.txt")?;
    manager.run();

    Ok(())
}
