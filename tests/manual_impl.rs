use futures::{future::BoxFuture, prelude::*};
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

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_actor_impl() {
    let mut actor = MyActor::default().spawn();

    for value in 1..10 {
        let id = actor
            .add_id(1)
            .await
            .expect("Failed to invoke `add_id` on actor");
        assert_eq!(id, value);
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
    type Result = usize;

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, Self::Result> {
        actor.add_id(self.0).boxed()
    }
}
