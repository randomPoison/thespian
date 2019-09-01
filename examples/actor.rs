use runtime::time::*;
use std::{sync::Arc, time::Duration};
use thespian::*;

#[runtime::main]
async fn main() {
    // Spawn the actor as a concurrent task.
    let (actor, context) = MyActor::default().into_context();
    runtime::spawn(context.run());

    // Communicate asynchronously with the actor from the current task, transparently
    // using message passing under the hood.
    loop {
        Delay::new(Duration::from_secs(3)).await;
        let id = actor
            .add_id(1)
            .await
            .expect("Failed to invoke `add_id` on actor");
        println!("New ID: {}", id);
    }
}

#[derive(Debug, Default)]
pub struct MyActor {
    name: Arc<String>,
    id: usize,
}

#[thespian::actor]
impl MyActor {
    pub fn name(&self) -> Arc<String> {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Arc::new(name);
    }

    pub async fn add_id(&mut self, value: usize) -> usize {
        self.id += value;
        self.id
    }
}
