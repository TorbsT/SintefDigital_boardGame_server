use serde::{Deserialize, Serialize};

use crate::game_data::custom_types::NodeID;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: NodeID,
    pub name: String,
    pub is_connected_to_rail: bool,
    pub is_parking_spot: bool,
}

impl Node {
    #[must_use]
    pub const fn new(id: NodeID, name: String) -> Self {
        Self {
            id,
            name,
            is_parking_spot: false,
            is_connected_to_rail: false,
        }
    }

    /// Toggles the is_connected_to_rail field.
    pub fn toggle_rail_connection(&mut self) {
        self.is_connected_to_rail = !self.is_connected_to_rail;
    }
}