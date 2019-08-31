use futures::{prelude::*, select};
use std::{future::Future, sync::Arc};
use thespian::*;

#[derive(Debug, Default)]
pub struct MyActor {
    id: usize,
}

impl MyActor {
    pub async fn add_id(&mut self, value: usize) -> usize {
        self.id += value;
        self.id
    }
}

// =================================================================================
// Generated from `thespian::actor`
// =================================================================================

impl Actor for MyActor {
    type Proxy = MyActorProxy;
}

#[derive(Debug, Clone)]
pub struct MyActorProxy {
    inner: ProxyFor<MyActor>,
}

impl MyActorProxy {
    pub async fn add_id(&mut self, value: usize) -> Result<usize, MessageError> {
        self.inner.send_async(MyActor__add_id(value)).await
    }
}

impl ActorProxy for MyActorProxy {
    type Actor = MyActor;

    fn new(inner: ProxyFor<MyActor>) -> Self {
        MyActorProxy { inner }
    }
}

#[derive(Debug)]
#[allow(bad_style)]
struct MyActor__add_id(usize);

impl AsyncMessage for MyActor__add_id {
    type Actor = MyActor;
    type Future = Box<dyn Future<Output = usize> + Unpin>;

    fn handle(self, actor: &mut Self::Actor) -> Self::Future {
        // Box::new(Box::pin(actor.add_id(self.0)))
        unimplemented!()
    }
}
