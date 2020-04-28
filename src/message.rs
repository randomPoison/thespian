//! Traits for defining actor messages.

use crate::Actor;
use futures::future::BoxFuture;

pub trait Message: 'static + Sized + Send {
    type Actor: Actor;
    type Output: Sized + Send;

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, Self::Output>;
}

pub trait ErasedMessage<A: Actor>: Send {
    fn handle(self: Box<Self>, actor: &mut A) -> BoxFuture<'_, ()>;
}
