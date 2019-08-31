//! Traits for defining actor messages.

use crate::Actor;
use std::future::Future;

pub trait SyncMessage: Sized {
    type Actor: Actor;
    type Result: Sized;

    fn handle(self, actor: &mut Self::Actor) -> Self::Result;
}

pub trait AsyncMessage: 'static + Sized {
    type Actor: Actor;
    type Future: Future;

    fn handle(self, actor: &mut Self::Actor) -> Self::Future;
}

pub trait SyncErasedMessage<A: Actor> {
    fn handle(self: Box<Self>, actor: &mut A);
}

pub trait AsyncErasedMessage<A: Actor> {
    fn handle(self: Box<Self>, actor: &mut A) -> Box<dyn Future<Output = ()> + Unpin + '_>;
}
