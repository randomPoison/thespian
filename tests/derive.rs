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
}
