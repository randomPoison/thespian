# Thespian

An experimental actor framework for [Rust](https://www.rust-lang.org/) with a focus on ergonomics and first class async/await support.

> If you need a real actor framework, you should check out [Actix](https://actix.rs/) instead. It provides a far more robust actor implementation and has direct support for building web severs.

## Current Status

Trying to build a proof-of-concept based on the following sketch:

```rust
use runtime::time::*;
use std::{sync::Arc, time::Duration};
use thespian::*;

#[runtime::main]
async fn main() {
    // Spawn the actor as a concurrent task.
    let actor = MyActor::default().start();

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
```

The goal is to create a very ergonomic way to communicate asynchronously with running actors by using code generation to hide all the boilerplate associated with message passing. If I can get the proof of concept working, I'll potentially flesh out the library and publish it.
