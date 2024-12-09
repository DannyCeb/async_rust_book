use futures_util::future::join_all;
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::prelude::*;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::task::JoinHandle;

type AsyncFileHandle = Arc<Mutex<File>>;
type FileJoinHandle = JoinHandle<Result<bool, String>>;

fn get_handle<T: ToString>(file_path: T) -> AsyncFileHandle {
    match OpenOptions::new().append(true).open(file_path.to_string()) {
        Ok(opened_file) => Arc::new(Mutex::new(opened_file)),
        Err(_) => Arc::new(Mutex::new(File::create(file_path.to_string()).unwrap())),
    }
}

struct AsyncWriteFuture {
    pub handle: AsyncFileHandle,
    pub entry: String,
}

impl Future for AsyncWriteFuture {
    type Output = Result<bool, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.handle.try_lock() {
            Ok(mut guard) => {
                let lined_entry = format!("{}\n", self.entry);
                match guard.write_all(lined_entry.as_bytes()) {
                    Ok(_) => println!("Written for: {}", self.entry),
                    Err(e) => println!("{}", e),
                }
                Poll::Ready(Ok(true))
            }
            Err(e) => {
                println!("error for: {} : {}", self.entry, e);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

fn write_log(file: AsyncFileHandle, line: String) -> FileJoinHandle {
    let awf = AsyncWriteFuture {
        handle: file,
        entry: line,
    };

    tokio::task::spawn(async move { awf.await })
}

#[tokio::main]
async fn main() {
    let login_handle = get_handle("login.txt");
    let logout_handle = get_handle("logout.txt");

    let names = ["one", "two", "three", "four", "five", "six"];
    let mut handles: Vec<JoinHandle<Result<bool, String>>> = Vec::new();

    for name in names {
        let file_handle = login_handle.clone();
        let file_handle_two = logout_handle.clone();
        let handle = write_log(file_handle, name.to_string());
        let handle_two = write_log(file_handle_two, name.to_string());
        handles.push(handle);
        handles.push(handle_two);
    }

    let _ = join_all(handles).await;
}
