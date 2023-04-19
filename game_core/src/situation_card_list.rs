use game_data::{SituationCard};

use crate::game_data;

//TODO: make list over all situation cards (See notes-resources channel for pictures of all situation cards)

pub fn situation_card_list() -> Vec<SituationCard> {
    let situation_card_list = vec![
        SituationCard::new(
            1,
            "title".to_string(),
            "description".to_string(),
            "goal".to_string(),
            Vec::new(),
        ),
    ];
    return situation_card_list;
}