use futures::{
    channel::oneshot,
    prelude::*,
    task::{Context, Poll},
};
use std::{future::Future, marker::PhantomData, ops::Deref, pin::Pin};

pub struct Addr<A: Actor> {
    proxy: A::Proxy,
}

impl<A: Actor> Deref for Addr<A> {
    type Target = A::Proxy;

    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

pub trait Actor: Sized {
    type Context: ActorContext;
    type Proxy: ActorProxy<Actor = Self>;

    fn start(self) -> Addr<Self> {
        unimplemented!()
    }
}

pub trait ActorContext {
    fn into_future(self) -> Box<dyn Future<Output = ()>>;
}

pub trait ActorProxy: Sized {
    type Actor: Actor<Proxy = Self>;
}

#[derive(Debug)]
pub struct MessageError;

pub struct MessageSender<M, R> {
    _marker: PhantomData<(M, R)>,
}

impl<M, R> MessageSender<M, R> {
    pub async fn send(&self, _message: M) -> Result<R, MessageError> {
        unimplemented!()
    }
}

pub struct MessageReceiver<M, R> {
    _marker: PhantomData<(M, R)>,
}

impl<M, R> Stream for MessageReceiver<M, R> {
    type Item = (M, oneshot::Sender<R>);

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}
