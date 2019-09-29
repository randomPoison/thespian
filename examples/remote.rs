use runtime::time::*;
use std::time::Duration;
use thespian::*;

#[runtime::main]
async fn main() {
    let (builder, remote) = StageBuilder::new();
    let actor = MyActor { remote, count: 0 };
    let stage = builder.finish(actor);
    let mut actor = stage.proxy();
    runtime::spawn(stage.run());

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
#[derive(Debug, Actor)]
pub struct MyActor {
    remote: Remote<Self>,
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
        Delay::new(Duration::from_secs(1)).await;
        self.count += value;

        if self.count >= 10 {
            let _ = self.remote.stop();
        }

        self.count
    }
}
