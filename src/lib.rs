use crate::envelope::*;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use log::*;

mod envelope;
mod message;

pub use crate::message::*;
pub use thespian_derive::actor;

pub trait Actor: 'static + Sized + Send {
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

    /// Spawns the actor onto the [runtime] threadpool.
    ///
    /// Returns the actor handle. Can only be used if the runtime has been initialized.
    /// If not using [runtime], or if you want more control over how the actor context is
    /// spawned, use [`into_context`] instead.
    ///
    /// [runtime]: https://docs.rs/runtime
    /// [`into_context`]: #tymethod.into_context
    fn spawn(self) -> Self::Proxy {
        let (proxy, context) = self.into_context();
        runtime::spawn(context.run());
        proxy
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

#[derive(Debug)]
pub struct MessageError {
    cause: MessageErrorCause,
}

impl From<oneshot::Canceled> for MessageError {
    fn from(_: oneshot::Canceled) -> Self {
        MessageError {
            cause: MessageErrorCause::ActorStopped,
        }
    }
}

impl From<mpsc::SendError> for MessageError {
    fn from(from: mpsc::SendError) -> Self {
        let cause = if from.is_full() {
            MessageErrorCause::MailboxFull
        } else if from.is_disconnected() {
            MessageErrorCause::ActorStopped
        } else {
            warn!("Unknown cause of send error: {:?}", from);
            MessageErrorCause::ActorStopped
        };

        MessageError { cause }
    }
}

#[derive(Debug)]
pub enum MessageErrorCause {
    MailboxFull,
    ActorStopped,
}
