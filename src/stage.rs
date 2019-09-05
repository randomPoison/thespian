use crate::{envelope::*, message::*, Actor, MessageError};
use derivative::Derivative;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use log::*;
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicU8, Arc},
};

pub struct StageBuilder<A: Actor> {
    remote: Arc<RemoteInner>,
    _marker: PhantomData<A>,
}

impl<A: Actor> StageBuilder<A> {
    pub fn new() -> (Self, Remote<A>) {
        unimplemented!()
    }

    pub fn finish(actor: A) -> Stage<A> {
        unimplemented!()
    }
}

pub struct Stage<A: Actor> {
    actor: A,
    stream: mpsc::Receiver<Envelope<A>>,

    /// Share a reference to the `RemoteInner` so that we can check the state.
    remote: Arc<RemoteInner>,
}

impl<A: Actor> Stage<A> {
    /// Consumes the stage, returning a future tha will run the actor until it is stopped.
    pub async fn run(mut self) {
        while let Some(envelope) = self.stream.next().await {
            match envelope {
                Envelope::Sync(message) => message.handle(&mut self.actor),
                Envelope::Async(message) => message.handle(&mut self.actor).await,
            }
        }
    }
}

pub trait ActorProxy: Sized + Clone {
    type Actor: Actor<Proxy = Self>;

    fn new(inner: ProxyFor<Self::Actor>) -> Self;
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub struct ProxyFor<A: Actor> {
    sink: mpsc::Sender<Envelope<A>>,
    proxy_count: Arc<()>,
}

impl<A: Actor> ProxyFor<A> {
    pub async fn send_sync<M: SyncMessage<Actor = A>>(
        &mut self,
        message: M,
    ) -> Result<M::Result, MessageError> {
        let (result_sender, result) = oneshot::channel();
        let erased_message = Box::new(SyncEnvelope {
            message,
            result_sender,
        });
        let envelope = Envelope::Sync(erased_message);
        self.sink
            .send(envelope)
            .await
            .map_err::<MessageError, _>(Into::into)?;
        result.await.map_err(Into::into)
    }

    pub async fn send_async<M: AsyncMessage<Actor = A>>(
        &mut self,
        message: M,
    ) -> Result<M::Result, MessageError> {
        let (result_sender, result) = oneshot::channel();
        let erased_message = Box::new(AsyncEnvelope {
            message,
            result_sender,
        });
        let envelope = Envelope::Async(erased_message);
        self.sink
            .send(envelope)
            .await
            .map_err::<MessageError, _>(Into::into)?;
        result.await.map_err(Into::into)
    }
}

/// Remote controller for an actor to manage its own state.
#[derive(Debug)]
pub struct Remote<A: Actor> {
    inner: Arc<RemoteInner>,
    proxy: ProxyFor<A>,
}

impl<A: Actor> Remote<A> {
    pub fn proxy(&self) -> A::Proxy {
        A::Proxy::new(self.proxy.clone())
    }
}

#[derive(Debug)]
struct RemoteInner {
    state: AtomicU8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ActorState {
    Building,
    Built,
    Running,
    Stopping,
    Stopped,
}
