use futures::future;
use thespian::*;

#[derive(Debug, Default)]
pub struct MyActor {
    id: usize,
}

#[thespian::actor]
impl MyActor {
    pub fn id(&self) -> usize {
        self.id
    }

    pub async fn add_id(&mut self, value: usize) -> usize {
        self.id += value;
        self.id
    }
}

// Test having multiple tasks communicate with an actor concurrently. This uses the
// default runtime implementation, which is a threadpool, so it also tests threading
// support.
#[runtime::test]
async fn multiple_tasks() {
    // Spawn the actor as a concurrent task.
    let (mut actor, context) = MyActor::default().into_context();
    runtime::spawn(context.run());

    // Spawn 10 tasks, each of which will add 10 to the actor's value.
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let mut actor = actor.clone();
        let join_handle = runtime::spawn(async move {
            for _ in 0..10 {
                actor
                    .add_id(1)
                    .await
                    .expect("Failed to invoke `add_id` on actor");
            }
        });
        tasks.push(join_handle);
    }

    future::join_all(tasks).await;
    assert_eq!(
        100,
        actor.id().await.expect("Failed to invoke `id` on actor")
    );
}
