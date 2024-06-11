use std::future::Future;

pub trait AwaitResult<T> {
    async fn await_result(self) -> T;
}

impl<F, T, E> AwaitResult<Result<T, E>> for Result<F, E>
where
    F: Future<Output = T> + Send,
{
    async fn await_result(self) -> Result<T, E> {
        Ok(self?.await)
    }
}

pub trait AsyncMap<T, F> {
    async fn async_map(self, f: F) -> T;
}

impl<F, U, R, T, E> AsyncMap<Result<R, E>, F> for Result<T, E>
where
    U: Future<Output = R> + Send,
    F: FnOnce(T) -> U,
{
    async fn async_map(self, f: F) -> Result<R, E> {
        match self {
            Ok(t) => Ok(f(t).await),
            Err(err) => Err(err),
        }
    }
}

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

pub trait FutureAndThen<T, F> {
    async fn and_then(self, f: F) -> T;
}

impl<F, I, R, T> FutureAndThen<R, F> for T
where
    T: Future<Output = I> + Send,
    F: FnOnce(I) -> R,
{
    async fn and_then(self, f: F) -> R {
        f(self.await)
    }
}
