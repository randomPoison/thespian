use crate::{envelope::*, message::*, remote::*, Actor, MessageError};
use derivative::Derivative;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use log::*;
use num_derive::{FromPrimitive, ToPrimitive};
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicU8, Arc},
};

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
/// struct MyActor {
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
    _marker: PhantomData<A>,
}

impl<A: Actor> StageBuilder<A> {
    pub fn new() -> (Self, Remote<A>) {
        let remote = Arc::new(RemoteInner::new(ActorState::Building));

        let (sender, receiver) = mpsc::channel(16);
        let remote_inner = Arc::new(RemoteInner::new(ActorState::Built));
        let proxy = ProxyFor {
            sink: sender,
            proxy_count: Arc::new(()),
        };

        let builder = Self {
            remote,
            receiver,
            _marker: Default::default(),
        };

        let remote = Remote {
            inner: remote,
            proxy: proxy,
        };
        (builder, remote)
    }

    pub fn finish(actor: A) -> Stage<A> {
        unimplemented!()
    }
}

pub struct Stage<A: Actor> {
    actor: A,
    stream: mpsc::Receiver<Envelope<A>>,

    /// Share a reference to the `RemoteInner` so that we can check the state.
    remote: Arc<RemoteInner>,
}

impl<A: Actor> Stage<A> {
    /// Consumes the stage, returning a future tha will run the actor until it is stopped.
    pub async fn run(mut self) {
        while let Some(envelope) = self.stream.next().await {
            match envelope {
                Envelope::Sync(message) => message.handle(&mut self.actor),
                Envelope::Async(message) => message.handle(&mut self.actor).await,
            }
        }
    }
}

pub trait ActorProxy: Sized + Clone {
    type Actor: Actor<Proxy = Self>;

    fn new(inner: ProxyFor<Self::Actor>) -> Self;
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub struct ProxyFor<A: Actor> {
    sink: mpsc::Sender<Envelope<A>>,
    proxy_count: Arc<()>,
}

impl<A: Actor> ProxyFor<A> {
    pub async fn send_sync<M: SyncMessage<Actor = A>>(
        &mut self,
        message: M,
    ) -> Result<M::Result, MessageError> {
        let (result_sender, result) = oneshot::channel();
        let erased_message = Box::new(SyncEnvelope {
            message,
            result_sender,
        });
        let envelope = Envelope::Sync(erased_message);
        self.sink
            .send(envelope)
            .await
            .map_err::<MessageError, _>(Into::into)?;
        result.await.map_err(Into::into)
    }

    pub async fn send_async<M: AsyncMessage<Actor = A>>(
        &mut self,
        message: M,
    ) -> Result<M::Result, MessageError> {
        let (result_sender, result) = oneshot::channel();
        let erased_message = Box::new(AsyncEnvelope {
            message,
            result_sender,
        });
        let envelope = Envelope::Async(erased_message);
        self.sink
            .send(envelope)
            .await
            .map_err::<MessageError, _>(Into::into)?;
        result.await.map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ActorState {
    Building,
    Built,
    Running,
    Stopping,
    Stopped,
}
