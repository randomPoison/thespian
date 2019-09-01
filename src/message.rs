//! Traits for defining actor messages.

use crate::Actor;
use futures::future::BoxFuture;

pub trait SyncMessage: 'static + Sized + Send {
    type Actor: Actor;
    type Result: Sized + Send;

    fn handle(self, actor: &mut Self::Actor) -> Self::Result;
}

pub trait AsyncMessage: 'static + Sized + Send {
    type Actor: Actor;
    type Result: Sized + Send;

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, Self::Result>;
}

pub trait SyncErasedMessage<A: Actor>: Send {
    fn handle(self: Box<Self>, actor: &mut A);
}

pub trait AsyncErasedMessage<A: Actor>: Send {
    fn handle(self: Box<Self>, actor: &mut A) -> BoxFuture<'_, ()>;
}
