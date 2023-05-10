use serde::{Deserialize, Serialize};

use crate::game_data::{custom_types::{PlayerID, GameID, NodeID, SituationCardID}, enums::{player_input_type::PlayerInputType, in_game_id::InGameID}};

use super::{district_modifier::DistrictModifier, edge_restriction::EdgeRestriction};

/// The PlayerInput struct describes the input of a player.
/// 
/// The option values should be set to something based on the input_type.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub player_id: PlayerID,
    pub game_id: GameID,
    pub input_type: PlayerInputType,
    pub related_role: Option<InGameID>,
    pub related_node_id: Option<NodeID>,
    pub district_modifier: Option<DistrictModifier>,
    pub situation_card_id: Option<SituationCardID>,
    pub edge_modifier: Option<EdgeRestriction>,
    pub related_bool: Option<bool>,
}