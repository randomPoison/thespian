use futures::{Future, FutureExt};

#[cfg(feature = "tokio")]
pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
{
    tokio::spawn(future.map(|_| {}));
}

#[cfg(feature = "async-std")]
pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
{
    async_std::task::spawn(future.map(|_| {}));
}
