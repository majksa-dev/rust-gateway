pub mod time;

use std::future::Future;

pub trait AsyncAndThen<T, F> {
    async fn async_and_then(self, f: F) -> T;
}

impl<F, U, R, T, E> AsyncAndThen<Result<R, E>, F> for Result<T, E>
where
    U: Future<Output = Result<R, E>> + Send,
    F: FnOnce(T) -> U,
{
    async fn async_and_then(self, f: F) -> Result<R, E> {
        match self {
            Ok(t) => f(t).await,
            Err(err) => Err(err),
        }
    }
}
