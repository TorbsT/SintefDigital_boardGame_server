use serde::{Deserialize, Serialize};

use crate::game_data::custom_types::SituationCardID;

use super::situation_card::SituationCard;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SituationCardList {
    pub situation_cards: Vec<SituationCard>,
}

impl SituationCardList {
    #[must_use]
    pub const fn new(situation_cards: Vec<SituationCard>) -> Self {
        Self { situation_cards }
    }

    pub fn get_default_situation_card_by_id(id: SituationCardID) -> Result<SituationCard, String> {
        let situation_cards = crate::situation_card_list::situation_card_list_wrapper();
        situation_cards
            .situation_cards
            .iter()
            .find(|card| card.card_id == id)
            .map_or_else(
                || Err(format!("There was no code with the ID: {}", id)),
                |card| Ok(card.clone()),
            )
    }
}