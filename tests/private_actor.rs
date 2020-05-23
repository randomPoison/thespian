//! This test verifies that an actor can be private to its crate/module without
//! generating a compiler error.

use thespian::Actor;

#[derive(Actor)]
struct PrivateActor {
    val: u32,
}

#[thespian::actor]
impl PrivateActor {
    pub fn add_to_val(&mut self, add: u32) {
        self.val += add;
    }
}
