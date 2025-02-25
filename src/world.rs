use gami_mc_protocol::registry::{Dimension, EntityKind};

#[derive(Default)]
pub struct World {
    pub dimension: Dimension,
    pub entities: Vec<EntityKind>,
}

impl World {
    pub fn get_entity(&mut self, id: i32) -> Option<&EntityKind> {
        self.entities.iter().find(|e| e.id() == id)
    }

    pub fn get_entity_mut(&mut self, id: i32) -> Option<&mut EntityKind> {
        self.entities.iter_mut().find(|e| e.id() == id)
    }
}
