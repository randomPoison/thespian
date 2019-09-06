use crate::{
    stage::{ActorState, ProxyFor},
    Actor, ActorProxy,
};
use std::sync::{atomic::AtomicU8, Arc};

/// Remote controller for an actor to manage its own state.
#[derive(Debug)]
pub struct Remote<A: Actor> {
    inner: Arc<RemoteInner>,
    proxy: ProxyFor<A>,
}

impl<A: Actor> Remote<A> {
    pub fn proxy(&self) -> A::Proxy {
        A::Proxy::new(self.proxy.clone())
    }
}

#[derive(Debug)]
pub(crate) struct RemoteInner {
    state: AtomicU8,
}

impl RemoteInner {
    pub(crate) fn new(state: ActorState) -> Self {
        Self {
            state: AtomicU8::new(state as u8),
        }
    }

    pub(crate) fn set_state(&self, state: ActorState) -> ActorState {
        self.state.swap(state as u8) as ActorState
    }
}
