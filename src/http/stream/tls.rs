use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

type Stream = Arc<Mutex<TlsStream<TcpStream>>>;

pub struct ReadHalf {
    inner: Stream,
}

pub struct WriteHalf {
    inner: Stream,
}

pub fn split(stream: TlsStream<TcpStream>) -> (ReadHalf, WriteHalf) {
    let stream = Arc::new(Mutex::new(stream));
    (
        ReadHalf {
            inner: stream.clone(),
        },
        WriteHalf { inner: stream },
    )
}

impl AsyncRead for ReadHalf {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut().inner.lock() {
            Ok(stream) => Pin::new(stream).as_mut().poll_read(cx, buf),
            Err(_) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to lock stream",
            ))),
        }
    }
}

impl AsyncWrite for WriteHalf {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut().inner.lock() {
            Ok(stream) => Pin::new(stream).as_mut().poll_write(cx, buf),
            Err(_) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to lock stream",
            ))),
        }
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut().inner.lock() {
            Ok(stream) => Pin::new(stream).as_mut().poll_write_vectored(cx, bufs),
            Err(_) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to lock stream",
            ))),
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.inner
            .lock()
            .map(|stream| stream.is_write_vectored())
            .unwrap_or_default()
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut().inner.lock() {
            Ok(stream) => Pin::new(stream).as_mut().poll_flush(cx),
            Err(_) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to lock stream",
            ))),
        }
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut().inner.lock() {
            Ok(stream) => Pin::new(stream).as_mut().poll_shutdown(cx),
            Err(_) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to lock stream",
            ))),
        }
    }
}
