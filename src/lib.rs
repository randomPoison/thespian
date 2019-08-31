use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
    task::Poll,
};
use std::{future::Future, ops::Deref};

mod envelope;
mod message;

pub use crate::{envelope::*, message::*};
pub use thespian_derive::actor;

pub trait Actor: Sized {
    type Proxy: ActorProxy<Actor = Self>;

    fn into_context(self) -> (Self::Proxy, Context<Self>) {
        // TODO: Make the channel buffer configurable.
        let (sender, receiver) = mpsc::channel(16);
        let proxy = Self::Proxy::new(ProxyFor { sink: sender });

        let context = Context {
            actor: self,
            stream: receiver,
        };

        (proxy, context)
    }
}

pub struct Context<A: Actor> {
    actor: A,
    stream: mpsc::Receiver<Envelope<A>>,
}

impl<A: Actor> Context<A> {
    /// Consumes the context, returning a future tha will run the actor until it is stopped.
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

#[derive(Debug)]
pub struct ProxyFor<A: Actor> {
    sink: mpsc::Sender<Envelope<A>>,
}

impl<A: Actor> Clone for ProxyFor<A> {
    fn clone(&self) -> Self {
        Self {
            sink: self.sink.clone(),
        }
    }
}

impl<A: Actor> ProxyFor<A> {
    pub async fn send_sync<M: SyncMessage>(
        &mut self,
        message: M,
    ) -> Result<M::Result, MessageError> {
        unimplemented!()
    }

    pub async fn send_async<M: AsyncMessage>(
        &mut self,
        message: M,
    ) -> Result<<M::Future as Future>::Output, MessageError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MessageError {
    cause: MessageErrorCause,
}

#[derive(Debug)]
pub enum MessageErrorCause {
    MailboxFull,
    ActorStopped,
}
