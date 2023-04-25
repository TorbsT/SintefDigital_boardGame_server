use game_data::{SituationCard};

use crate::game_data::{self, Neighbourhood};

pub fn situation_card_list() -> Vec<SituationCard> {
    let situation_card_list = vec![
        SituationCard::new(
            1,
            "Regular traffic".to_string(),
            "Regular traffic in all zones.".to_string(),
            "Facilitate transport operations. Rewards green behavior.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Trafic::LevelOne),
                (Neighbourhood::Suburbs, Trafic::LevelOne),
                (Neighbourhood::Port, Trafic::LevelOne),
                (Neighbourhood::RingRoad, Trafic::LevelOne),
                (Neighbourhood::CityCentre, Trafic::LevelOne),
                (Neighbourhood::Airport, Trafic::LevelOne),
            ],
        ),
        SituationCard::new(
            2,
            "Concert".to_string(),
            "City centre is crowded. Reduced capacity for traffic.".to_string(),
            "Facilitate transport of people to concert. Limit other traffic in city centre to what is necesary.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Trafic::LevelOne),
                (Neighbourhood::Suburbs, Trafic::LevelOne),
                (Neighbourhood::Port, Trafic::LevelOne),
                (Neighbourhood::RingRoad, Trafic::LevelThree),
                (Neighbourhood::CityCentre, Trafic::LevelFive),
                (Neighbourhood::Airport, Trafic::LevelOne),
            ],
        ),
        SituationCard::new(
            3,
            "Gas Leakage".to_string(),
            "Gas leakage in Industry Park zone. Health and explosion risk.".to_string(),
            "Evacuate people and dangerous goods from the area. Safety comes first.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Trafic::LevelOne),
                (Neighbourhood::Suburbs, Trafic::LevelOne),
                (Neighbourhood::Port, Trafic::LevelOne),
                (Neighbourhood::RingRoad, Trafic::LevelThree),
                (Neighbourhood::CityCentre, Trafic::LevelOne),
                (Neighbourhood::Airport, Trafic::LevelOne),
            ],
        ),
        SituationCard::new(
            4,
            "Accident".to_string(),
            "Accident in ring road section I6 - I7. Traffic blocked in east-bound lanes".to_string(),
            "Support emergency services. Coordinate with other zones.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Trafic::LevelOne),
                (Neighbourhood::Suburbs, Trafic::LevelOne),
                (Neighbourhood::Port, Trafic::LevelThree),
                (Neighbourhood::RingRoad, Trafic::LevelFive),
                (Neighbourhood::CityCentre, Trafic::LevelThree),
                (Neighbourhood::Airport, Trafic::LevelOne),
            ],
        ),
        SituationCard::new(
            5,
            "Airport train stops".to_string(),
            "No train from City Centre to Airport during rush hours. Delays for passengers.".to_string(),
            "Passengers reach airport in time.".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Trafic::LevelOne),
                (Neighbourhood::Suburbs, Trafic::LevelTwo),
                (Neighbourhood::Port, Trafic::LevelOne),
                (Neighbourhood::RingRoad, Trafic::LevelFour),
                (Neighbourhood::CityCentre, Trafic::LevelOne),
                (Neighbourhood::Airport, Trafic::LevelFour),
            ],
        ),
    ];
    return situation_card_list;
}