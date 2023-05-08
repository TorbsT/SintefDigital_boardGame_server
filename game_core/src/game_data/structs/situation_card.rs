use serde::{Deserialize, Serialize};

use crate::game_data::custom_types::SituationCardID;

use super::{cost_tuple::CostTuple, player_objective_card::PlayerObjectiveCard};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SituationCard {
    pub card_id: SituationCardID,
    pub title: String,
    pub description: String,
    pub goal: String,
    pub costs: Vec<CostTuple>,
    pub objective_cards: Vec<PlayerObjectiveCard>,
}

impl SituationCard {
    #[must_use]
    pub const fn new(
        card_id: SituationCardID,
        title: String,
        description: String,
        goal: String,
        costs: Vec<CostTuple>,
        objective_cards: Vec<PlayerObjectiveCard>,
    ) -> Self {
        Self {
            card_id,
            title,
            description,
            goal,
            costs,
            objective_cards,
        }
    }
}