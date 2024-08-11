use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::{client, server, TlsStream};

// #[derive(Debug)]
// pub enum Stream {
//     Client(client::TlsStream<TcpStream>),
//     Server(server::TlsStream<TcpStream>),
// }

// impl AsyncRead for Stream {
//     #[inline]
//     fn poll_read(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &mut ReadBuf<'_>,
//     ) -> Poll<io::Result<()>> {
//         match self.get_mut() {
//             Stream::Client(stream) => Pin::new(stream).as_mut().poll_read(cx, buf),
//             Stream::Server(stream) => Pin::new(stream).as_mut().poll_read(cx, buf),
//         }
//     }
// }

// impl AsyncWrite for Stream {
//     #[inline]
//     fn poll_write(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &[u8],
//     ) -> Poll<io::Result<usize>> {
//         match self.get_mut() {
//             Stream::Client(stream) => Pin::new(stream).as_mut().poll_write(cx, buf),
//             Stream::Server(stream) => Pin::new(stream).as_mut().poll_write(cx, buf),
//         }
//     }

//     #[inline]
//     fn poll_write_vectored(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         bufs: &[io::IoSlice<'_>],
//     ) -> Poll<io::Result<usize>> {
//         match self.get_mut() {
//             Stream::Client(stream) => Pin::new(stream).as_mut().poll_write_vectored(cx, bufs),
//             Stream::Server(stream) => Pin::new(stream).as_mut().poll_write_vectored(cx, bufs),
//         }
//     }

//     #[inline]
//     fn is_write_vectored(&self) -> bool {
//         match self {
//             Stream::Client(stream) => stream.is_write_vectored(),
//             Stream::Server(stream) => stream.is_write_vectored(),
//         }
//     }

//     #[inline]
//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
//         match self.get_mut() {
//             Stream::Client(stream) => Pin::new(stream).as_mut().poll_flush(cx),
//             Stream::Server(stream) => Pin::new(stream).as_mut().poll_flush(cx),
//         }
//     }

//     #[inline]
//     fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
//         match self.get_mut() {
//             Stream::Client(stream) => Pin::new(stream).as_mut().poll_shutdown(cx),
//             Stream::Server(stream) => Pin::new(stream).as_mut().poll_shutdown(cx),
//         }
//     }
// }

#[derive(Debug)]
pub struct ReadHalf {
    inner: Arc<Mutex<TlsStream<TcpStream>>>,
}

#[derive(Debug)]
pub struct WriteHalf {
    inner: Arc<Mutex<TlsStream<TcpStream>>>,
}

pub fn split_client(stream: client::TlsStream<TcpStream>) -> (ReadHalf, WriteHalf) {
    split(stream.into())
}

pub fn split_server(stream: server::TlsStream<TcpStream>) -> (ReadHalf, WriteHalf) {
    split(stream.into())
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
