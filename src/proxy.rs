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

    /// Sends a message to an actor.
    ///
    /// If the actor is still running and there is space in its message queue, the
    /// message will be enqueued synchronously. Otherwise, an error will be returned.
    pub fn send_message<M: Message<Actor = A>>(&mut self, message: M) -> Result<(), MessageError> {
        let erased_message = Box::new(message);
        let envelope = Envelope::Message(erased_message);
        self.sink.try_send(envelope).map_err(Into::into)
    }

    /// Sends a request to an actor, returning a future yielding the actor's response.
    ///
    /// If the actor has stopped or its message queue is full, this method will return
    /// an error synchronously. Otherwise, the message will be queued and the returned
    /// future will resolve to the actor's response.
    ///
    /// If the actor panics while handling the message, the panic will be propagated to
    /// any code awaiting the response.
    pub fn send_request<R: Request<Actor = A>>(
        &mut self,
        message: R,
    ) -> Result<impl Future<Output = R::Result>, MessageError> {
        let (result_sender, result) = oneshot::channel();
        let erased_message = Box::new(RequestEnvelope {
            message,
            result_sender,
        });
        let envelope = Envelope::Message(erased_message);
        self.sink.try_send(envelope)?;

        // Message was successfully enqueued. Return a future that awaits the message
        // response and panics if the actor failed to one, since the only case where an
        // actor wouldn't send a response is if it panics while handling the request.
        Ok(async { result.await.expect("Actor panicked while handling message") })
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
        // Manually drop the inner ref count in order to ensure the count has decreased
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
