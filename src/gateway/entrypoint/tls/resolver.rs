use std::sync::Arc;

use tokio_rustls::rustls::{
    server::{ClientHello, ResolvesServerCert},
    sign,
};

/// A resolver that always returns `None`.
#[derive(Debug)]
pub struct EmptyResolver;

impl EmptyResolver {
    pub fn new() -> Self {
        Self
    }
}

impl ResolvesServerCert for EmptyResolver {
    fn resolve(&self, _client_hello: ClientHello) -> Option<Arc<sign::CertifiedKey>> {
        None
    }
}
