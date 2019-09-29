use crate::{envelope::*, message::*, Actor, MessageError};
use derivative::Derivative;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use std::{
    mem,
    sync::{Arc, Weak},
};

pub(crate) type EnvelopeSender<A> = mpsc::Sender<Envelope<A>>;

pub trait ActorProxy: Sized + Clone {
    type Actor: Actor<Proxy = Self>;

    fn new(inner: ProxyFor<Self::Actor>) -> Self;
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub struct ProxyFor<A: Actor> {
    sink: EnvelopeSender<A>,

    // NOTE: We wrap the ref count in an `Option` in order to control the drop order.
    // On drop, we send a message to the stage, but we need to ensure that the ref
    // count has been decremented before the message is received. Wrapping it in an
    // `Option` means we can `take` the value to drop it early. The proxy count will
    // always have a value outside of the destructor, so it's safe to unwrap.
    proxy_count: Option<Arc<()>>,
}

impl<A: Actor> ProxyFor<A> {
    pub(crate) fn new(sink: EnvelopeSender<A>) -> Self {
        Self {
            sink,
            proxy_count: Some(Arc::new(())),
        }
    }

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

    pub(crate) fn count(&self) -> usize {
        Arc::strong_count(self.proxy_count.as_ref().unwrap())
    }

    pub(crate) fn downgrade(&self) -> WeakProxyFor<A> {
        WeakProxyFor {
            sink: self.sink.clone(),
            proxy_count: Arc::downgrade(self.proxy_count.as_ref().unwrap()),
        }
    }
}

impl<A: Actor> Drop for ProxyFor<A> {
    fn drop(&mut self) {
        // Manaully drop the inner ref count in order to ensure the count has decreased
        // *before* the stage receives the drop message.
        mem::drop(self.proxy_count.take());

        // Send the drop message so that the stage can stop itself if there are no
        // proxies left.
        //
        // NOTE: We don't care if the message send fails here in the case that the buffer
        // is full. If that happens, the stage will check the proxy count after processing
        // the next message to see if there are any proxies left.
        let _ = self.sink.try_send(Envelope::ProxyDropped);
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub(crate) struct WeakProxyFor<A: Actor> {
    sink: EnvelopeSender<A>,
    proxy_count: Weak<()>,
}

impl<A: Actor> WeakProxyFor<A> {
    pub(crate) fn upgrade(&self) -> Option<ProxyFor<A>> {
        self.proxy_count.upgrade().map(|proxy_count| ProxyFor {
            sink: self.sink.clone(),
            proxy_count: Some(proxy_count),
        })
    }
}
