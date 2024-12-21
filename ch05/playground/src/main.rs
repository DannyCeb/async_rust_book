#![feature(coroutines)]
#![feature(coroutine_trait)]

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader};

use std::io::Write;
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

fn main() -> io::Result<()> {
    Ok(())
}
