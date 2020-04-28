use futures::{future::BoxFuture, prelude::*};
use thespian::*;

#[derive(Debug, Default)]
pub struct MyActor {
    value: usize,
}

// #[thespian::actor]
impl MyActor {
    pub fn value(&self) -> usize {
        self.value
    }

    pub async fn add_async(&mut self, value: usize) -> usize {
        self.value += value;
        self.value
    }

    pub fn add_sync(&mut self, value: usize) -> usize {
        self.value += value;
        self.value
    }

    pub fn add(&mut self, value: usize) {
        self.value += value;
    }
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_actor_impl() {
    let mut actor = MyActor::default().spawn();

    for value in 1..10 {
        let result = actor.add_sync(1).unwrap().await;
        assert_eq!(value, result);
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
    pub fn value(&mut self) -> thespian::Result<impl Future<Output = usize>> {
        self.inner.send_request(MyActor_value())
    }

    pub fn add_sync(&mut self, value: usize) -> thespian::Result<impl Future<Output = usize>> {
        self.inner.send_request(MyActor__add_sync(value))
    }

    pub fn add_async(&mut self, value: usize) -> thespian::Result<impl Future<Output = usize>> {
        self.inner.send_request(MyActor__add_async(value))
    }

    pub fn add(&mut self, value: usize) -> thespian::Result<()> {
        self.inner.send_message(MyActor__add(value))
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
struct MyActor_value();

impl Message for MyActor_value {
    type Actor = MyActor;
    type Output = usize;

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, Self::Output> {
        async move { actor.value() }.boxed()
    }
}

#[derive(Debug)]
#[allow(bad_style)]
struct MyActor__add_sync(usize);

impl Message for MyActor__add_sync {
    type Actor = MyActor;
    type Output = usize;

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, Self::Output> {
        async move { actor.add_sync(self.0) }.boxed()
    }
}

#[derive(Debug)]
#[allow(bad_style)]
struct MyActor__add_async(usize);

impl Message for MyActor__add_async {
    type Actor = MyActor;
    type Output = usize;

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, Self::Output> {
        async move { actor.add_async(self.0).await }.boxed()
    }
}

#[derive(Debug)]
#[allow(bad_style)]
struct MyActor__add(usize);

impl Message for MyActor__add {
    type Actor = MyActor;
    type Output = ();

    fn handle(self, actor: &mut Self::Actor) -> BoxFuture<'_, ()> {
        async move { actor.add(self.0) }.boxed()
    }
}
