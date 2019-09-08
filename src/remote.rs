use crate::{
    stage::{ActorState, ProxyFor},
    Actor, ActorProxy,
};
use std::{
    convert::TryInto,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

/// Remote controller for an actor to manage its own state.
#[derive(Debug)]
pub struct Remote<A: Actor> {
    pub(crate) inner: Arc<RemoteInner>,
    pub(crate) proxy: ProxyFor<A>,
}

impl<A: Actor> Remote<A> {
    pub fn proxy(&self) -> A::Proxy {
        A::Proxy::new(self.proxy.clone())
    }

    pub fn stop(&self) -> Result<(), StopError> {
        unimplemented!()
    }

    pub fn state(&self) -> ActorState {
        self.inner.state()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StopError;

#[derive(Debug)]
pub(crate) struct RemoteInner {
    state: AtomicU8,
}

impl RemoteInner {
    pub(crate) fn new(state: ActorState) -> Self {
        Self {
            state: AtomicU8::new(state.into()),
        }
    }

    pub(crate) fn set_state(&self, state: ActorState) -> ActorState {
        self.state
            .swap(state as u8, Ordering::SeqCst)
            .try_into()
            .expect("Failed to convert raw actor state")
    }

    pub(crate) fn state(&self) -> ActorState {
        self.state
            .load(Ordering::SeqCst)
            .try_into()
            .expect("Failed to convert raw actor state")
    }
}
