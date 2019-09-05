use crate::{envelope::*, stage::*};
use derivative::Derivative;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use log::*;
use std::sync::{atomic::AtomicU8, Arc};

mod envelope;
mod message;
mod stage;

pub use crate::message::*;
pub use thespian_derive::actor;

pub trait Actor: 'static + Sized + Send {
    type Proxy: ActorProxy<Actor = Self>;

    fn into_stage(self) -> (Self::Proxy, Stage<Self>) {
        // TODO: Make the channel buffer configurable.
        let (sender, receiver) = mpsc::channel(16);
        let remote_inner = Arc::new(RemoteInner::new(ActorState::Built));
        let proxy = Self::Proxy::new(ProxyFor {
            sink: sender,
            proxy_count: Arc::new(()),
        });

        let stage = Stage {
            actor: self,
            remote: remote_inner,
            stream: receiver,
        };

        (proxy, stage)
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
