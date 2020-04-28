#![allow(unused_imports)]

use futures::future;
use thespian::*;

#[derive(Debug, Default, Actor)]
pub struct Counter {
    value: usize,
}

#[thespian::actor]
impl Counter {
    pub fn value(&self) -> usize {
        self.value
    }

    pub async fn add(&mut self, value: usize) -> usize {
        self.value += value;
        self.value
    }
}

// Test having multiple tasks communicate with an actor concurrently. This uses the
// default runtime implementation, which is a thread pool, so it also tests threading
// support.
#[cfg(feature = "tokio")]
#[tokio::test]
async fn multiple_tasks() {
    // Spawn the actor as a concurrent task.
    let mut actor = Counter::default().spawn();

    // Spawn 10 tasks, each of which will add 10 to the actor's value.
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let mut actor = actor.clone();
        let join_handle = tokio::spawn(async move {
            for _ in 0..10 {
                actor.add(1).unwrap().await;
            }
        });
        tasks.push(join_handle);
    }

    future::join_all(tasks).await;
    assert_eq!(100, actor.value().unwrap().await);
}
