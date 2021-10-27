use futures::{Future, Sink, SinkExt};
use std::fmt;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::{JoinError, JoinHandle};

pub async fn catch_error<Fut, E>(future: Fut)
where
    Fut: Future<Output=Result<(), E>>,
    E: fmt::Display + fmt::Debug
{
    let res = future.await;
    match res {
        Ok(()) => {},
        Err(e) => {
            log::error!("{}", e);
            log::debug!("Details: {:?}", e)
        }
    }
}

pub struct KillJoinHandle<T> {
    handle: Option<JoinHandle<T>>
}

impl<T> KillJoinHandle<T> {
    pub fn bg(mut self) -> JoinHandle<T> {
        self.handle.take().unwrap()
    }
}

impl<T> Drop for KillJoinHandle<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl<T> Future for KillJoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref mut handle) = self.handle {
            JoinHandle::poll(Pin::new(handle), cx)
        } else {
            panic!("Future has already been consumed");
        }
    }
}

pub fn kill_task_on_drop<T>(handle: JoinHandle<T>) -> KillJoinHandle<T> {
    KillJoinHandle { handle: Some(handle) }
}