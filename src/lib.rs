use crate::stage::*;
use futures::channel::{mpsc, oneshot};
use log::*;

mod envelope;
mod message;
mod remote;
mod stage;

pub use crate::{message::*, remote::Remote};
pub use thespian_derive::actor;

pub trait Actor: 'static + Sized + Send {
    type Proxy: ActorProxy<Actor = Self>;

    fn into_stage(self) -> Stage<Self> {
        let (builder, _) = StageBuilder::new();
        builder.finish(self)
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
        let stage = self.into_stage();
        let proxy = stage.proxy();
        runtime::spawn(stage.run());
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
