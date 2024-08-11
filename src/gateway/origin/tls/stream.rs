use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tokio_rustls::TlsStream;

use crate::http::stream::Split;

pub enum Stream {
    Tcp(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Split<ReadHalf, WriteHalf> for Stream {
    fn to_split(self) -> (ReadHalf, WriteHalf) {
        match self {
            Stream::Tcp(stream) => {
                let (rx, tx) = stream.to_split();
                (ReadHalf::Tcp(rx), WriteHalf::Tcp(tx))
            }
            Stream::Tls(stream) => {
                let (rx, tx) = stream.to_split();
                (ReadHalf::Tls(rx), WriteHalf::Tls(tx))
            }
        }
    }
}

#[derive(Debug)]
pub enum WriteHalf {
    Tcp(OwnedWriteHalf),
    Tls(crate::WriteHalf),
}

impl AsyncWrite for WriteHalf {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            WriteHalf::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
            WriteHalf::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            WriteHalf::Tcp(stream) => Pin::new(stream).poll_write_vectored(cx, bufs),
            WriteHalf::Tls(stream) => Pin::new(stream).poll_write_vectored(cx, bufs),
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        match self {
            WriteHalf::Tcp(stream) => Pin::new(stream).is_write_vectored(),
            WriteHalf::Tls(stream) => Pin::new(stream).is_write_vectored(),
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            WriteHalf::Tcp(stream) => Pin::new(stream).poll_flush(cx),
            WriteHalf::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            WriteHalf::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
            WriteHalf::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

#[derive(Debug)]
pub enum ReadHalf {
    Tcp(OwnedReadHalf),
    Tls(crate::ReadHalf),
}

impl AsyncRead for ReadHalf {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            ReadHalf::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
            ReadHalf::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}
