#[cfg(feature = "tls")]
pub mod tls;

use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

#[cfg(feature = "tls")]
pub type ReadHalf = tls::ReadHalf;
#[cfg(not(feature = "tls"))]
pub type ReadHalf = OwnedReadHalf;

#[cfg(feature = "tls")]
pub type WriteHalf = tls::WriteHalf;
#[cfg(not(feature = "tls"))]
pub type WriteHalf = OwnedWriteHalf;

pub trait Split<R, W> {
    fn to_split(self) -> (R, W);
}

impl Split<OwnedReadHalf, OwnedWriteHalf> for TcpStream {
    fn to_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
        self.into_split()
    }
}

#[cfg(feature = "tls")]
impl Split<tls::ReadHalf, tls::WriteHalf> for tokio_rustls::server::TlsStream<TcpStream> {
    fn to_split(self) -> (tls::ReadHalf, tls::WriteHalf) {
        tls::split(self)
    }
}
