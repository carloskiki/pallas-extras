use futures::{AsyncRead, AsyncWrite, FutureExt, lock::Mutex};
use std::{
    collections::VecDeque,
    io::{self, Read as _, Write},
    ops::DerefMut,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker, ready},
};

#[derive(Debug)]
pub struct DuplexHandle {
    #[allow(clippy::type_complexity)]
    buffers: Option<
        Arc<(
            Mutex<(VecDeque<u8>, Option<Waker>)>,
            Mutex<(VecDeque<u8>, Option<Waker>)>,
        )>,
    >,
    tx_first: bool,
}

impl AsyncRead for DuplexHandle {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let Some((first, second)) = self.buffers.as_deref() else {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Handle is closed",
            )));
        };

        let rx_lock = if self.tx_first { second } else { first };
        let mut lock = ready!(rx_lock.lock().poll_unpin(cx));
        let (ref mut rx, ref mut waker) = *lock;
        if rx.is_empty() {
            if let Some(waker) = waker {
                waker.clone_from(cx.waker());
            } else {
                *waker = Some(cx.waker().clone());
            }
            return Poll::Pending;
        }

        Poll::Ready(rx.read(buf))
    }
}

impl AsyncWrite for DuplexHandle {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let Some((first, second)) = self.buffers.as_deref() else {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "other end closed",
            )));
        };
        let rx_lock = if self.tx_first { first } else { second };
        let mut lock = ready!(rx_lock.lock().poll_unpin(cx));
        let (ref mut tx, ref mut waker) = *lock;

        // One can't reaquire a duplex handle so if the count is 1 we know the other end is closed
        if Arc::strong_count(
            self.buffers
                .as_ref()
                .expect("should be Some as we already checked."),
        ) == 1
        {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "other end closed",
            )));
        }
        let result = tx.write(buf);
        if let Some(waker) = waker.take() {
            waker.wake();
        }
        Poll::Ready(result)
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.deref_mut().buffers = None;
        Poll::Ready(Ok(()))
    }
}

pub fn full_duplex() -> (DuplexHandle, DuplexHandle) {
    let buffers = Arc::new((
        Mutex::new((VecDeque::new(), None)),
        Mutex::new((VecDeque::new(), None)),
    ));

    let a = DuplexHandle {
        buffers: Some(Arc::clone(&buffers)),
        tx_first: true,
    };
    let b = DuplexHandle {
        buffers: Some(Arc::clone(&buffers)),
        tx_first: false,
    };
    (a, b)
}

