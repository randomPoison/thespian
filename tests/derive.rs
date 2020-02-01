#![allow(clippy::blacklisted_name)]

use thespian::Actor;

#[derive(Debug, Actor)]
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

    pub fn multiple_params(&self, _first: usize, _second: String) {
        unimplemented!()
    }

    pub fn mut_param(&self, mut some_param: String) -> String {
        some_param.push_str(" and some more stuff");
        some_param
    }

    pub fn pat_param_tuple(&self, (foo, bar): (usize, String)) {
        dbg!(foo, bar);
    }
}

// Test that the derive works with a second impl block.
#[thespian::actor]
impl MyActor {
    pub fn do_thing(&self) {
        unimplemented!()
    }
}

mod submodule {
    use super::{MyActor, MyActorProxy};

    #[thespian::actor]
    impl MyActor {
        pub fn another_thing(&self) {
            unimplemented!()
        }
    }
}
