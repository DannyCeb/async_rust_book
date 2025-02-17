use std::{
    future::Future,
    io::{self, Read},
    net::TcpStream,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

pub struct TcpReceiver {
    pub stram: Arc<Mutex<TcpStream>>,
    pub buffer: Vec<u8>,
}

impl Future for TcpReceiver {
    type Output = io::Result<Vec<u8>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut stream = match self.stram.try_lock() {
            Ok(stream) => stream,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        let mut local_buff = [0; 2014];

        match stream.read(&mut local_buff) {
            Ok(0) => Poll::Ready(Ok(self.buffer.to_vec())),
            Ok(n) => {
                std::mem::drop(stream);
                self.buffer.extend_from_slice(&local_buff[..n]);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
