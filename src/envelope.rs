//! Functionality for wrapping messages into type-erased envelopes.
//!
//! When messages are sent to actors, two things must happen:
//!
//! * They must be type-erased so that different message types can be sent through
//!   the same channel.
//! * The message must be bundled with oneshot channel in order to send the message
//!   response back to the sender.

use crate::{Actor, ErasedMessage, Message};
use futures::{channel::oneshot, future::BoxFuture, prelude::*};
use std::fmt;

pub(crate) enum Envelope<A: Actor> {
    Message(Box<dyn ErasedMessage<A>>),
    ProxyDropped,
}

impl<A: Actor> fmt::Debug for Envelope<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Envelope::Message(..) => write!(f, "Envelope::Message"),
            Envelope::ProxyDropped => write!(f, "Envelope::ProxyDropped"),
        }
    }
}

impl<M: Message> ErasedMessage<M::Actor> for M {
    fn handle(self: Box<Self>, actor: &mut M::Actor) -> BoxFuture<'_, ()> {
        // TODO: Remove the extra boxing here. In theory, we should be able to constrain
        // this impl to only messages where `Output == ()`, but that's not currently
        // supported. See https://github.com/rust-lang/rust/issues/20041 for more
        // information.
        Message::handle(*self, actor).map(|_| {}).boxed()
    }
}

pub(crate) struct RequestEnvelope<M: Message> {
    pub(crate) result_sender: oneshot::Sender<M::Output>,
    pub(crate) message: M,
}

impl<M: Message> ErasedMessage<M::Actor> for RequestEnvelope<M> {
    fn handle(self: Box<Self>, actor: &mut M::Actor) -> BoxFuture<'_, ()> {
        async move {
            let result = self.message.handle(actor).await;

            // If the message sender has dropped the handle the attempt to send the result will
            // fail. In that cases, there's nothing we can reasonably do other than discard the
            // result.
            let _ = self.result_sender.send(result);
        }
        .boxed()
    }
}
