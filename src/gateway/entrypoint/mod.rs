mod service;
#[cfg(not(feature = "tls"))]
pub mod tcp;
#[cfg(feature = "tls")]
pub mod tls;

use std::sync::Arc;

use crate::{Middleware, ReadHalf, WriteHalf};

pub use service::EntryPoint;

pub type MiddlewaresItem = Arc<dyn Middleware + Send + Sync + 'static>;

pub type Middlewares<'a> = Box<dyn Iterator<Item = MiddlewaresItem> + Send + Sync + 'a>;
