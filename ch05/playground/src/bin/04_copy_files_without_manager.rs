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

trait SymetricCoroutine {
    type Input;
    type Output;

    fn resume_with_output(self: Pin<&mut Self>, input: Self::Input) -> Self::Output;
}

impl SymetricCoroutine for ReadCoroutine {
    type Input = ();
    type Output = Option<i32>;

    fn resume_with_output(mut self: Pin<&mut Self>, _input: Self::Input) -> Self::Output {
        if let Some(Ok(line)) = self.lines.next() {
            line.parse::<i32>().ok()
        } else {
            None
        }
    }
}

impl SymetricCoroutine for WriteCoroutine {
    type Input = i32;
    type Output = ();

    fn resume_with_output(mut self: Pin<&mut Self>, input: i32) -> Self::Output {
        writeln!(self.file_handle, "{}", input).unwrap();
    }
}

fn main() -> io::Result<()> {
    let mut reader = ReadCoroutine::new("numbers.txt")?;
    let mut writer = WriteCoroutine::new("numbers_copy.txt")?;

    loop {
        match Pin::new(&mut reader).resume_with_output(()) {
            Some(number) => {
                Pin::new(&mut writer).resume_with_output(number);
            }
            None => {
                break;
            }
        }
    }

    Ok(())
}
