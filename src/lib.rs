use futures::channel::{mpsc, oneshot};
use log::*;
use thiserror::Error;

mod envelope;
mod message;
mod proxy;
mod remote;
mod stage;

// Helper module for abstracting over different runtimes.
#[cfg(any(feature = "tokio", feature = "async-std"))]
mod runtime;

pub use crate::{message::*, proxy::*, remote::*, stage::*};
pub use thespian_derive::*;

pub trait Actor: 'static + Sized + Send {
    type Proxy: ActorProxy<Actor = Self>;

    fn into_stage(self) -> Stage<Self> {
        let (builder, _) = StageBuilder::new();
        builder.finish(self)
    }

    /// Spawns the actor onto the [runtime] thread pool.
    ///
    /// Returns the actor handle. If one of the runtime features is enabled, i.e. either
    /// "tokio" or "async-std". If not using one of the supported runtimes, or if you
    /// want more control over how the actor context is spawned, use [`into_context`]
    /// instead.
    ///
    /// [runtime]: https://docs.rs/runtime
    /// [`into_context`]: #tymethod.into_context
    #[cfg(any(feature = "tokio", feature = "async-std"))]
    fn spawn(self) -> Self::Proxy {
        let stage = self.into_stage();
        let proxy = stage.proxy();

        // Spawn the actor using the selected runtime.
        crate::runtime::spawn(stage.run());

        proxy
    }
}

#[derive(Debug, Clone, Error)]
#[error("{cause}")]
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

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum MessageErrorCause {
    #[error("Message box was full")]
    MailboxFull,

    #[error("Actor was stopped")]
    ActorStopped,

    #[error("Unknown reason for message error")]
    Unknown,
}
