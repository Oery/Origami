use std::collections::HashMap;

use gami_mc_protocol::registry::{Dimension, EntityKind};

#[derive(Default)]
pub struct World {
    pub dimension: Dimension,
    pub entities: HashMap<i32, EntityKind>,
}
