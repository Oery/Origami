use anyhow::anyhow;
use gami_mc_protocol::registry::Dimension;

use crate::entity::Entity;

#[derive(Default)]
pub struct World {
    pub dimension: Dimension,
    pub entities: Vec<Entity>,
}

impl World {
    pub fn spawn_entity(&mut self, id: i32) {
        if !self.entities.iter().any(|e| e.id == id) {
            self.entities.push(Entity::new(id))
        }
    }

    pub fn get_entity(&mut self, id: i32) -> anyhow::Result<&mut Entity> {
        if !self.entities.iter().any(|e| e.id == id) {
            self.entities.push(Entity::new(id))
        }

        self.entities
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or(anyhow!("Failed to find Entity"))
    }
}
