use serde::{Deserialize, Serialize};

use crate::game_data::{custom_types::{NodeID, VehicleType}, enums::{restriction_type::RestrictionType, type_entities_to_transport::TypeEntitiesToTransport}, constants::HEAVY_VEHICLE_INCLUSIVE_THRESHOLD};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PlayerObjectiveCard {
    pub name: String,
    pub start_node_id: NodeID,
    pub pick_up_node_id: NodeID,
    pub drop_off_node_id: NodeID,
    pub special_vehicle_types: Vec<RestrictionType>,
    pub picked_package_up: bool,
    pub dropped_package_off: bool,
    pub type_of_entities_to_transport: TypeEntitiesToTransport,
    pub amount_of_entities: u32,
}

impl PlayerObjectiveCard {
    /// Creates a new PlayerObjectiveCard and adds the Heavy vehicle type if the amount of entities to transport is greater than or equal to the HEAVY_VEHICLE_INCLUSIVE_THRESHOLD.
    pub fn new(
        name: String,
        start_node_id: NodeID,
        pick_up_node_id: NodeID,
        drop_off_node_id: NodeID,
        vehicle_types: Vec<VehicleType>,
        type_of_entities_to_transport: TypeEntitiesToTransport,
        amount_of_entities: u32,
    ) -> Self {
        let mut special_vehicle_types = vehicle_types;

        if amount_of_entities >= HEAVY_VEHICLE_INCLUSIVE_THRESHOLD {
            special_vehicle_types.push(VehicleType::Heavy);
        }

        Self {
            start_node_id,
            pick_up_node_id,
            drop_off_node_id,
            special_vehicle_types,
            picked_package_up: false,
            dropped_package_off: false,
            name,
            amount_of_entities,
            type_of_entities_to_transport,
        }
    }
}