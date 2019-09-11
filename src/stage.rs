use crate::{envelope::*, proxy::*, remote::*, Actor};
use futures::{channel::mpsc, prelude::*};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{marker::PhantomData, sync::Arc};

/// Builder for initializing an actor that needs its own [`Remote`].
///
/// In order to stop itself and access its own proxy, an actor uses its [`Remote`].
/// The remote it created along with the [`Stage`] that runs the actor. This poses
/// a problem during initialization, though: You need to have created the actor
/// object in order to initialize the stage, but you need the stage in order to get
/// the remote needed to initialize the actor. In order to resolve this,
/// `StageBuilder` provides a way to get a [`Remote`] without having to fully create
/// a [`Stage`] first.
///
/// # Examples
///
/// ```
/// use thespian::{Remote, StageBuilder};
///
/// pub struct MyActor {
///     remote: Remote<Self>
/// }
///
/// #[thespian::actor]
/// impl MyActor {}
///
/// let (builder, remote) = StageBuilder::new();
/// let actor = MyActor { remote };
/// let stage = builder.finish(actor);
/// ```
///
/// [`Remote`]: struct.Remote.html
pub struct StageBuilder<A: Actor> {
    remote: Arc<RemoteInner>,
    receiver: mpsc::Receiver<Envelope<A>>,
    proxy: ProxyFor<A>,
    _marker: PhantomData<A>,
}

impl<A: Actor> StageBuilder<A> {
    pub fn new() -> (Self, Remote<A>) {
        let remote_inner = Arc::new(RemoteInner::new(ActorState::Building));

        let (sender, receiver) = mpsc::channel(16);
        let proxy = ProxyFor::new(sender);

        let remote = Remote::new(remote_inner.clone(), &proxy);

        let builder = Self {
            remote: remote_inner,
            receiver,
            proxy: proxy.clone(),
            _marker: Default::default(),
        };

        (builder, remote)
    }

    pub fn finish(self, actor: A) -> Stage<A> {
        Stage {
            actor,
            receiver: self.receiver,
            proxy: self.proxy,
            remote: self.remote,
        }
    }
}

pub struct Stage<A: Actor> {
    actor: A,
    receiver: mpsc::Receiver<Envelope<A>>,

    // Hold onto a proxy for the actor.
    //
    // NOTE: We can't hold a `WeakProxyFor<A>` here because that would mean that, once
    // all externally-held proxies had been dropped, there would be no way to construct
    // a new proxy for the actor, since the inner proxy count has been dropped. Since
    // the remote only holds a weak reference to the proxy count, this also ensures that
    // a new proxy can't be created once the actor has been stopped and the stage has
    // been dropped.
    proxy: ProxyFor<A>,

    /// Share a reference to the `RemoteInner` so that we can check the state.
    remote: Arc<RemoteInner>,
}

impl<A: Actor> Stage<A> {
    /// Consumes the stage, returning a future tha will run the actor until it is stopped.
    pub async fn run(mut self) {
        // Mark that the actor is running.
        //
        // TODO: Do we have to do anything here to handle the case where the actor has
        // already been asked to stop? Is that a valid case, or would we reject any stop
        // requests that come in before the actor has started running?
        self.remote.set_state(ActorState::Running);

        // TODO: What would it mean for `stream.next()` to return `None` here? Since the stage
        // holds onto a copy of the proxy, that case should never happen right?
        while let Some(envelope) = self.receiver.next().await {
            match envelope {
                Envelope::Sync(message) => message.handle(&mut self.actor),
                Envelope::Async(message) => message.handle(&mut self.actor).await,

                // NOTE: We don't need to do anything in the case that a proxy was dropped, since
                // we check the proxy count at the end of the loop body.
                Envelope::ProxyDropped => {}
            }

            // Check if the actor has stopped itself after each message we process.
            //
            // TODO: Do we need to be able to stop the actor while still waiting for a message?
            // It's technically valid currently for the actor to hand off its remote to another
            // task/thread that could stop the actor at an arbitrary time, though doing so is
            // not the intended use case.
            if self.remote.state() == ActorState::Stopping {
                break;
            }

            // Check if there are any proxies held by other tasks. As long as the actor is running
            // there will be at least one proxy, since the stage holds onto one itself. If the
            // count drops to one, that means no other tasks are holding onto proxies and we
            // therefore cannot receive any new messages.
            if self.proxy.count() == 1 {
                break;
            }
        }

        // Close the channel so that no new messages can be sent.
        self.receiver.close();

        // Process any remaining messages.
        while let Some(envelope) = self.receiver.next().await {
            match envelope {
                Envelope::Sync(message) => message.handle(&mut self.actor),
                Envelope::Async(message) => message.handle(&mut self.actor).await,
                Envelope::ProxyDropped => {}
            }
        }

        // Mark that the actor has fully stopped.
        self.remote.set_state(ActorState::Stopped);
    }

    pub fn proxy(&self) -> A::Proxy {
        A::Proxy::new(self.proxy.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ActorState {
    Building,
    Built,
    Running,
    Stopping,
    Stopped,
}
