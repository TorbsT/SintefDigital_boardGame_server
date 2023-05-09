use serde::{Deserialize, Serialize};

use crate::game_data::{custom_types::{NodeID, MovementCost}, enums::{district::District, restriction_type::RestrictionType}};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NeighbourRelationship {
    pub to: NodeID,
    pub neighbourhood: District,
    pub movement_cost: MovementCost,
    pub blocked: bool,
    pub is_connected_through_rail: bool,
    pub restriction: Option<RestrictionType>,
    pub is_modifiable: bool,
}

impl NeighbourRelationship {
    pub const fn new(
        to: NodeID,
        neighbourhood: District,
        movement_cost: MovementCost,
        is_connected_through_rail: bool,
    ) -> Self {
        let blocked = false;
        Self {
            to,
            neighbourhood,
            movement_cost,
            blocked,
            is_connected_through_rail,
            restriction: None,
            is_modifiable: true,
        }
    }
}