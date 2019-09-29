# Thespian

An experimental actor framework for [Rust](https://www.rust-lang.org/) with a focus on ergonomics and first class async/await support.

> NOTE: If you need a real actor framework, you should check out [Actix](https://actix.rs/) instead. It provides a far more robust actor implementation and has direct support for building web severs.

# Example Usage

```rust
use runtime::time::*;
use std::time::Duration;
use thespian::*;

#[derive(Debug, Default, Actor)]
pub struct MyActor {
    count: usize,
}

// The `thespian::actor` attribute makes all methods defined
// in this impl block available as messages on the actor.
#[thespian::actor]
impl MyActor {
    pub async fn add_count(&mut self, value: usize) -> usize {
        // Simulate a slow, asynchronous operation, such as
        // writing to a database.
        Delay::new(Duration::from_secs(1)).await;

        self.count += value;
        self.count
    }
}

#[runtime::main]
async fn main() {
    let mut handle = MyActor::default().spawn();

    for _ in 0..10 {
        let id = handle
            .add_count(1)
            .await
            .expect("Failed to invoke `add_id` on actor");
        println!("New count: {}", id);
    }
}
```

## Current Status

The basic functionality for defining actors and their messages is in place, as well as a rudimentary implementation of the actor runtime. The next steps are to expand and polish the library in various ways:

* Flesh out the actor API, e.g. support stopping actors and actor lifecycles.
* Improve code generation to make it more robust and flexible.
* Test the library in more real-world use cases to ensure it provides the necessary functionality.
* Benchmark and optimize the actor implementation.
