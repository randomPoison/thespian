use std::time::Duration;
use thespian::*;
use tokio::time;

#[tokio::main]
async fn main() {
    // Spawn the actor as a task on the default runtime. This returns a handle to
    // the actor that we can use to communicate with it from other tasks.
    let mut actor = MyActor::default().spawn();

    // Use the handle to call the `add_count` method. Under the hood, this is using
    // channels and message passing to communicate between tasks/threads, but
    // thespian hides those implementation details and provides a simple, await-aware
    // way to communicate with the actor.
    for _ in 0..10 {
        let id = actor
            .add_count(1)
            .await
            .expect("Failed to invoke `add_id` on actor");
        println!("New count: {}", id);
    }
}

// Actors are defined as normal structs.
#[derive(Debug, Default, Actor)]
pub struct MyActor {
    count: usize,
}

// To define messages for an actor, mark an impl bloc with the `thespian::actor`
// attribute. All methods defined in this impl block become messages that can be
// sent via the generated actor handle.
#[thespian::actor]
impl MyActor {
    /// Adds to the actor's count, simulating a slow operation such as writing to a
    /// database.
    pub async fn add_count(&mut self, value: usize) -> usize {
        self.count += value;
        time::delay_for(Duration::from_secs(1)).await;
        self.count
    }
}
