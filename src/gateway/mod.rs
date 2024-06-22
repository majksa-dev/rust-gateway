pub(crate) mod entrypoint;
pub mod middleware;
pub mod next;
pub mod origin;
pub mod router;

pub use next::Next;

pub type Result<T> = anyhow::Result<T>;
