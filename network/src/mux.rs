use std::{
    convert::Infallible,
    io,
    pin::pin,
    sync::{Arc, Mutex},
};

use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite,
    future::RemoteHandle,
    task::{Spawn, SpawnExt},
};

use super::Request;

#[derive(Debug, Clone)]
pub struct Mux<W> {
    writer: W,
    handle: Arc<Mutex<RemoteHandle<io::Result<Infallible>>>>,
}

impl<W: AsyncWrite> Mux<W> {
    pub fn new(
        reader: impl AsyncRead + Send + 'static,
        writer: W,
        spawner: impl Spawn,
    ) -> Result<Self, futures::task::SpawnError> {
        let handle = Arc::new(Mutex::new(spawner.spawn_with_handle(reader_task(reader))?));

        Ok(Self { writer, handle })
    }

    pub async fn write<'a, M>(&'a mut self, message: &M) -> io::Result<M::Response<'a>>
    where
        M: Request,
    {
        todo!()
    }
}

async fn reader_task<R: AsyncRead>(read: R) -> io::Result<Infallible> {
    let mut pinned_read = pin!(read);
    let mut header_buf = [0; 8];

    loop {
        pinned_read.read_exact(&mut header_buf).await?;
    }
}
