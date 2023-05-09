use serde::{Deserialize, Serialize};

use crate::game_data::{custom_types::NodeID, enums::restriction_type::RestrictionType};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EdgeRestriction {
    pub node_one: NodeID,
    pub node_two: NodeID,
    pub edge_restriction: RestrictionType,
    pub delete: bool,
}

impl EdgeRestriction {
    pub const fn new(node_id_one: NodeID, node_id_two: NodeID, edge_restriction: RestrictionType) -> Self {
        Self {
            node_one: node_id_one,
            node_two: node_id_two,
            delete: false,
            edge_restriction,
        }
    }
}