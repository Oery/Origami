#[derive(Debug, PartialEq)]
pub struct Entity {
    pub id: i32,
    pub coords: Option<Coordinates>,
}

impl Entity {
    pub fn new(id: i32) -> Entity {
        Entity { id, coords: None }
    }
}

#[derive(Debug, PartialEq)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: i8,
    pub pitch: i8,
}
