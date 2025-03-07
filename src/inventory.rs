use std::array;

use gami_mc_protocol::packets::play::server::Item;

#[derive(Debug)]
pub struct Inventory {
    pub slots: [Option<Item>; 45],
    pub carried: Option<Item>,
    pub main_hand: i8,
}

impl Inventory {
    const HOTBAR_START: usize = 36;
    const HOTBAR_END: usize = 44;

    pub fn hotbar(&self) -> &[Option<Item>] {
        &self.slots[Self::HOTBAR_START..=Self::HOTBAR_END]
    }

    pub fn hotbar_mut(&mut self) -> &mut [Option<Item>] {
        &mut self.slots[Self::HOTBAR_START..=Self::HOTBAR_END]
    }

    pub fn armor_slots(&self) -> &[Option<Item>] {
        &self.slots[5..=8]
    }

    pub fn armor_slots_mut(&mut self) -> &mut [Option<Item>] {
        &mut self.slots[5..=8]
    }

    pub fn main_hand(&self) -> &Option<Item> {
        self.hotbar()
            .get(self.main_hand as usize)
            .expect("Index out of bounds")
    }

    pub fn set_slot(&mut self, _slot: i8) {
        // TODO: Send relevant packet
        todo!()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: array::from_fn(|_| None),
            carried: None,
            main_hand: 0,
        }
    }
}
