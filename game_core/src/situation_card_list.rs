use game_data::{SituationCard};

use crate::game_data;

//TODO: make list over all situation cards (See notes-resources channel for pictures of all situation cards)

pub fn situation_card_list() -> Vec<SituationCard> {
    let situation_card_list = vec![
        SituationCard::new( //Template card; delete this after list is complete
            0,
            "title".to_string(),
            "description".to_string(),
            "goal".to_string(),
            Vec::new(),
        ),
        SituationCard::new(
            1,
            "Regular traffic".to_string(),
            "Regular traffic in all zones.".to_string(),
            "Facilitate transport operations. Rewards green behavior.".to_string(),
            Vec::new(),
        ),
        SituationCard::new(
            2,
            "Concert".to_string(),
            "City centre is crowded. Reduced capacity for traffic.".to_string(),
            "Facilitate transport of people to concert. Limit other traffic in city centre to what is necesary.".to_string(),
            Vec::new(), //ring road: 3, city centre: 5
        ),
        SituationCard::new(
            3,
            "Gas Leakage".to_string(),
            "Gas leakage in Industry Park zone. Health and explosion risk.".to_string(),
            "Evacuate people and dangerous goods from the area. Safety comes first.".to_string(),
            Vec::new(), //Industry Park: 1, Ring Road: 3
        ),
        SituationCard::new(
            4,
            "Accident".to_string(),
            "Accident in ring road section I6 - I7. Traffic blocked in east-bound lanes".to_string(),
            "Support emergency services. Coordinate with other zones.".to_string(),
            Vec::new(), //Ring Road: 5, City Centre: 3, Suburbs: 3
        ),
        SituationCard::new(
            5,
            "Airport train stops".to_string(),
            "No train from City Centre to Airport during rush hours. Delays for passengers.".to_string(),
            "Passengers reach airport in time.".to_string(),
            Vec::new(), //Airport: 4, Ring road: 4, Suburbs: 2
        ),
    ];
    return situation_card_list;
}