use serde::{Deserialize, Serialize};

use crate::game_data::{custom_types::{GameID, PlayerID, MovesRemaining, NodeID}, enums::in_game_id::InGameID};

use super::player_objective_card::PlayerObjectiveCard;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub connected_game_id: Option<GameID>,
    pub in_game_id: InGameID,
    pub unique_id: PlayerID,
    pub name: String,
    pub position_node_id: Option<NodeID>,
    pub remaining_moves: MovesRemaining,
    pub objective_card: Option<PlayerObjectiveCard>,
    pub is_bus: bool,
}

impl Player {
    #[must_use]
    pub const fn new(unique_id: PlayerID, name: String) -> Self {
        let is_bus = false;
        Self {
            connected_game_id: None,
            in_game_id: InGameID::Undecided,
            unique_id,
            name,
            position_node_id: None,
            remaining_moves: 0,
            objective_card: None,
            is_bus,
        }
    }

    pub fn transform_to_bus(&mut self) {
        self.is_bus = true;
    }

    pub fn transform_to_car(&mut self) {
        self.is_bus = false;
    }

}