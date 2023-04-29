use game_data::{SituationCard, SituationCardList};

use crate::game_data::{self, Neighbourhood, NodeMap, PlayerObjectiveCard, Traffic, VehicleType};

pub fn situation_card_list_wrapper() -> SituationCardList {
    SituationCardList::new(situation_card_list())
}

pub fn situation_card_list() -> Vec<SituationCard> {
    let map = NodeMap::new_default();
    vec![
        SituationCard::new(
            1,
            "Regular traffic".to_string(),
            "Regular traffic in all zones.".to_string(),
            "Facilitate transport operations. Rewards green behavior.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Traffic::LevelOne),
                (Neighbourhood::Suburbs, Traffic::LevelOne),
                (Neighbourhood::Port, Traffic::LevelOne),
                (Neighbourhood::RingRoad, Traffic::LevelOne),
                (Neighbourhood::CityCentre, Traffic::LevelOne),
                (Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new(13, 7, 15, Vec::new()),
                PlayerObjectiveCard::new(8, 11, 27, vec![VehicleType::Electric]),
                PlayerObjectiveCard::new(15, 23, 2, Vec::new()),
                PlayerObjectiveCard::new(17, 22, 14, vec![VehicleType::Electric]),
                PlayerObjectiveCard::new(5, 12, 28, Vec::new()),
                PlayerObjectiveCard::new(11, 14, 24, Vec::new()),
            ],
        ),
        SituationCard::new(
            2,
            "Concert".to_string(),
            "City centre is crowded. Reduced capacity for traffic.".to_string(),
            "Facilitate transport of people to concert. Limit other traffic in city centre to what is necesary.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Traffic::LevelOne),
                (Neighbourhood::Suburbs, Traffic::LevelOne),
                (Neighbourhood::Port, Traffic::LevelOne),
                (Neighbourhood::RingRoad, Traffic::LevelThree),
                (Neighbourhood::CityCentre, Traffic::LevelFive),
                (Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new(8, 14, 12, Vec::new()),
                PlayerObjectiveCard::new(14, 28, 12, Vec::new()),
                PlayerObjectiveCard::new(24, 22, 12, Vec::new()),
                PlayerObjectiveCard::new(22, 10, 12, vec![VehicleType::Electric]),
                PlayerObjectiveCard::new(5, 13, 28, Vec::new()),
                PlayerObjectiveCard::new(23, 10, 2, Vec::new()),
            ]
        ),
        SituationCard::new(
            3,
            "Gas Leakage".to_string(),
            "Gas leakage in Industry Park zone. Health and explosion risk.".to_string(),
            "Evacuate people and dangerous goods from the area. Safety comes first.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Traffic::LevelOne),
                (Neighbourhood::Suburbs, Traffic::LevelOne),
                (Neighbourhood::Port, Traffic::LevelOne),
                (Neighbourhood::RingRoad, Traffic::LevelThree),
                (Neighbourhood::CityCentre, Traffic::LevelOne),
                (Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new(4, 0, 10, vec![VehicleType::Emergency]),
                PlayerObjectiveCard::new(9, 0, 17, vec![VehicleType::Industrial, VehicleType::Emergency]),
                PlayerObjectiveCard::new(15, 0, 15, vec![VehicleType::Emergency]),
                PlayerObjectiveCard::new(5, 1, 17, vec![VehicleType::Industrial, VehicleType::Emergency]),
                PlayerObjectiveCard::new(24, 22, 10, Vec::new()),
                PlayerObjectiveCard::new(5, 5, 23, Vec::new()),
            ]
        ),
        SituationCard::new(
            4,
            "Accident".to_string(),
            "Accident in ring road section I6 - I7. Traffic blocked in east-bound lanes".to_string(),
            "Support emergency services. Coordinate with other zones.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Traffic::LevelOne),
                (Neighbourhood::Suburbs, Traffic::LevelOne),
                (Neighbourhood::Port, Traffic::LevelThree),
                (Neighbourhood::RingRoad, Traffic::LevelFive),
                (Neighbourhood::CityCentre, Traffic::LevelThree),
                (Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new(15, 19, 14, vec![VehicleType::Emergency]),
                PlayerObjectiveCard::new(14, 19, 14, vec![VehicleType::Emergency]),
                PlayerObjectiveCard::new(16, 16, 28, Vec::new()),
                PlayerObjectiveCard::new(17, 20, 28, vec![VehicleType::Electric]),
                PlayerObjectiveCard::new(27, 27, 15, vec![VehicleType::Electric]),
                PlayerObjectiveCard::new(23, 24, 7, Vec::new()),
            ]
        ),
        SituationCard::new(
            5,
            "Airport train stops".to_string(),
            "No train from City Centre to Airport during rush hours. Delays for passengers.".to_string(),
            "Passengers reach airport in time.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Traffic::LevelOne),
                (Neighbourhood::Suburbs, Traffic::LevelTwo),
                (Neighbourhood::Port, Traffic::LevelOne),
                (Neighbourhood::RingRoad, Traffic::LevelFour),
                (Neighbourhood::CityCentre, Traffic::LevelOne),
                (Neighbourhood::Airport, Traffic::LevelFour),
            ],
            vec![
                PlayerObjectiveCard::new(23, 10, 27, vec![VehicleType::Electric]),
                PlayerObjectiveCard::new(0, 2, 27, Vec::new()),
                PlayerObjectiveCard::new(5, 7, 28, Vec::new()),
                PlayerObjectiveCard::new(16, 10, 28, Vec::new()),
                PlayerObjectiveCard::new(14, 10, 27, Vec::new()),
                PlayerObjectiveCard::new(23, 24, 8, Vec::new()),
            ]
        ),
    ]
}
