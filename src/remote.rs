use crate::{
    proxy::{ProxyFor, WeakProxyFor},
    stage::ActorState,
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
    inner: Arc<RemoteInner>,
    proxy: WeakProxyFor<A>,
}

impl<A: Actor> Remote<A> {
    pub(crate) fn new(inner: Arc<RemoteInner>, proxy: &ProxyFor<A>) -> Self {
        Self {
            inner,
            proxy: proxy.downgrade(),
        }
    }

    /// Returns a proxy to the actor for this remote.
    ///
    /// # Panics
    ///
    /// This method will panic if the actor is no longer running and all other proxies
    /// for the actor have been dropped. A `Remote` should not outlive the actor it is
    /// tied to, so this is not a supported use case.
    pub fn proxy(&self) -> A::Proxy {
        let proxy = self
            .proxy
            .upgrade()
            .expect("Unable to get proxy from actor remote, did your `Remote` outlive your actor?");
        A::Proxy::new(proxy)
    }

    pub fn stop(&self) -> Result<(), StopError> {
        loop {
            let state = self.inner.state();
            match state {
                ActorState::Running => {
                    let result = self.inner.state.compare_and_swap(
                        state.into(),
                        ActorState::Stopping.into(),
                        Ordering::SeqCst,
                    );

                    if result != ActorState::Stopping.into() {
                        continue;
                    }
                }

                ActorState::Building | ActorState::Built => {
                    return Err(StopError);
                }

                ActorState::Stopping | ActorState::Stopped => {
                    return Ok(());
                }
            }
        }
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
            .swap(state.into(), Ordering::SeqCst)
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
