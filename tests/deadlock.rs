//! Test to verify that actors can send a message to an actor that is currently
//! processing a message without deadlocking.
//!
//! This tests the issue reported in [#9]. The test defines two actors `Foo` and
//! `Bar` that each hold on to the other's proxy. To trigger the potential deadlock
//! `Foo` sends a message to `Bar` from within one of its message handlers, which in
//! turn sends a message back to `Foo` while it's still processing the initial
//! message. In the original bug, both actors were waiting for the other to finish
//! processing their messages, leading to a deadlock.
//!
//! [#9]: https://github.com/randomPoison/thespian/issues/9

#![allow(unused_imports)]

use futures::{channel::oneshot, future};
use std::time::Duration;
use thespian::*;

#[derive(Debug, Actor)]
pub struct Foo {
    bar: BarProxy,
    result: Option<oneshot::Sender<usize>>,
}

#[thespian::actor]
impl Foo {
    pub async fn tell_bar(&mut self) {
        self.bar.add_to_foo().unwrap().await;
    }

    pub fn add(&mut self, value: usize) {
        self.result.take().unwrap().send(value).unwrap();
    }
}

#[derive(Debug, Actor)]
pub struct Bar {
    value: usize,
    foo: FooProxy,
}

#[thespian::actor]
impl Bar {
    pub fn add_to_foo(&mut self) -> bool {
        self.foo.add(self.value).unwrap();
        true
    }

    pub fn checkpoint(&self) -> () {}
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn possible_deadlock() {
    // Define the expected value for `Foo` to send, and created the channel it will use
    // to send it.
    let expected = 123;
    let (sender, receiver) = oneshot::channel();

    // Create both actors, giving each one a proxy for the other.
    let (foo_stage, foo_remote) = StageBuilder::new();
    let (bar_stage, bar_remote) = StageBuilder::new();
    foo_stage.spawn(Foo {
        result: Some(sender),
        bar: bar_remote.proxy(),
    });
    bar_stage.spawn(Bar {
        value: expected,
        foo: foo_remote.proxy(),
    });

    // Send the initial message to `Foo` to start the potential deadlock.
    let mut foo = foo_remote.proxy();
    foo.tell_bar().unwrap();

    // Request the updated value from `foo`. If the actors have deadlocked the timeout
    // will fire instead.
    let actual = tokio::time::timeout(Duration::from_millis(500), receiver)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(expected, actual);
}
