//! Functionality for wrapping messages into type-erased envelopes.
//!
//! When messages are sent to actors, two things must happen:
//!
//! * They must be type-erased so that different message types can be sent through
//!   the same channel.
//! * The message must be bundled with oneshot channel in order to send the message
//!   response back to the sender.

use crate::{Actor, AsyncErasedMessage, AsyncMessage, SyncErasedMessage, SyncMessage};
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
    task::Poll,
};
use std::{fmt, future::Future, marker::PhantomData, ops::Deref, pin::Pin, sync::Arc};

pub enum Envelope<A: Actor> {
    Sync(Box<dyn SyncErasedMessage<A>>),
    Async(Box<dyn AsyncErasedMessage<A>>),
}

impl<A: Actor> fmt::Debug for Envelope<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Envelope::Sync(..) => write!(f, "Envelope::Sync"),
            Envelope::Async(..) => write!(f, "Evenlope::Async"),
        }
    }
}

pub struct SyncEnvelope<M: SyncMessage> {
    result_sender: oneshot::Sender<M::Result>,
    message: M,
}

impl<M: SyncMessage> SyncErasedMessage<M::Actor> for SyncEnvelope<M> {
    fn handle(self: Box<Self>, actor: &mut M::Actor) {
        let result = self.message.handle(actor);

        // If the message sender has dropped the handle the attempt to send the result will
        // fail. In that cases, there's nothing we can reasonably do other than discard the
        // result.
        let _ = self.result_sender.send(result);
    }
}

pub struct AsyncEnvelope<M: AsyncMessage> {
    result_sender: oneshot::Sender<<M::Future as Future>::Output>,
    message: M,
}

impl<M: AsyncMessage> AsyncErasedMessage<M::Actor> for AsyncEnvelope<M> {
    fn handle(self: Box<Self>, actor: &mut M::Actor) -> Box<dyn Future<Output = ()> + Unpin + '_> {
        // TODO: Can we get this to work without doing `Box::new(Box::pin(...))`?
        Box::new(Box::pin(async move {
            let result = self.message.handle(actor).await;

            // If the message sender has dropped the handle the attempt to send the result will
            // fail. In that cases, there's nothing we can reasonably do other than discard the
            // result.
            let _ = self.result_sender.send(result);
        }))
    }
}
