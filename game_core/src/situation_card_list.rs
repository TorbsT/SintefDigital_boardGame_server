use game_data::{SituationCard, SituationCardList};

use crate::game_data::{self, CostTuple, Neighbourhood, PlayerObjectiveCard, Traffic, VehicleType, TypeEntitiesToTransport};

pub fn situation_card_list_wrapper() -> SituationCardList {
    SituationCardList::new(situation_card_list())
}

pub fn situation_card_list() -> Vec<SituationCard> {
    vec![
        SituationCard::new(
            1,
            "Regular traffic".to_string(),
            "Regular traffic in all zones.".to_string(),
            "Facilitate transport operations. Rewards green behavior.".to_string(),
            vec![
                CostTuple::new(Neighbourhood::IndustryPark, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Suburbs, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Port, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::RingRoad, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::CityCentre, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new("Packages".to_string(), 13, 7, 15, Vec::new(), TypeEntitiesToTransport::Packages, 5),
                PlayerObjectiveCard::new("Passengers".to_string(), 8, 11, 27, vec![VehicleType::Electric], TypeEntitiesToTransport::People, 3),
                PlayerObjectiveCard::new("Passengers".to_string(), 15, 23, 2, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Passengers".to_string(), 17, 22, 14, vec![VehicleType::Electric], TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Passengers".to_string(), 5, 12, 28, Vec::new(), TypeEntitiesToTransport::People, 3),
                PlayerObjectiveCard::new("Passengers".to_string(), 11, 14, 24, Vec::new(), TypeEntitiesToTransport::People, 3),
            ],
        ),
        SituationCard::new(
            2,
            "Concert".to_string(),
            "City centre is crowded. Reduced capacity for traffic.".to_string(),
            "Facilitate transport of people to concert. Limit other traffic in city centre to what is necesary.".to_string(),
            vec![
                CostTuple::new(Neighbourhood::IndustryPark, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Suburbs, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Port, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::RingRoad, Traffic::LevelThree),
                CostTuple::new(Neighbourhood::CityCentre, Traffic::LevelFive),
                CostTuple::new(Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new("Passengers".to_string(), 8, 14, 12, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Passengers".to_string(), 14, 28, 12, Vec::new(), TypeEntitiesToTransport::People, 5),
                PlayerObjectiveCard::new("Passengers".to_string(), 24, 22, 12, Vec::new(), TypeEntitiesToTransport::People, 5),
                PlayerObjectiveCard::new("Passengers".to_string(), 22, 10, 12, vec![VehicleType::Electric], TypeEntitiesToTransport::People, 3),
                PlayerObjectiveCard::new("Passengers".to_string(), 5, 13, 28, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Packages".to_string(), 23, 10, 2, Vec::new(), TypeEntitiesToTransport::Packages, 5),
            ]
        ),
        SituationCard::new(
            3,
            "Gas Leakage".to_string(),
            "Gas leakage in Industry Park zone. Health and explosion risk.".to_string(),
            "Evacuate people and dangerous goods from the area. Safety comes first.".to_string(),
            vec![
                CostTuple::new(Neighbourhood::IndustryPark, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Suburbs, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Port, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::RingRoad, Traffic::LevelThree),
                CostTuple::new(Neighbourhood::CityCentre, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new("Evacuate".to_string(), 4, 0, 10, vec![VehicleType::Emergency], TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Dangerous goods".to_string(), 9, 0, 17, vec![VehicleType::Hazard, VehicleType::Emergency], TypeEntitiesToTransport::Packages, 4),
                PlayerObjectiveCard::new("Ambulance".to_string(), 15, 0, 15, vec![VehicleType::Emergency], TypeEntitiesToTransport::People, 2),
                PlayerObjectiveCard::new("Evacuate".to_string(), 5, 1, 17, vec![VehicleType::Hazard, VehicleType::Emergency], TypeEntitiesToTransport::Packages, 3),
                PlayerObjectiveCard::new("Passengers".to_string(), 24, 22, 10, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Packages".to_string(), 5, 5, 23, Vec::new(), TypeEntitiesToTransport::Packages, 5),
            ]
        ),
        SituationCard::new(
            4,
            "Accident".to_string(),
            "Accident in ring road section I6 - I7. Traffic blocked in east-bound lanes".to_string(),
            "Support emergency services. Coordinate with other zones.".to_string(),
            vec![
                CostTuple::new(Neighbourhood::IndustryPark, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Suburbs, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Port, Traffic::LevelThree),
                CostTuple::new(Neighbourhood::RingRoad, Traffic::LevelFive),
                CostTuple::new(Neighbourhood::CityCentre, Traffic::LevelThree),
                CostTuple::new(Neighbourhood::Airport, Traffic::LevelOne),
            ],
            vec![
                PlayerObjectiveCard::new("Ambulance".to_string(), 15, 19, 14, vec![VehicleType::Emergency], TypeEntitiesToTransport::People, 1),
                PlayerObjectiveCard::new("Car removal".to_string(), 14, 19, 14, vec![VehicleType::Emergency], TypeEntitiesToTransport::Packages, 1),
                PlayerObjectiveCard::new("Passengers".to_string(), 16, 16, 28, Vec::new(), TypeEntitiesToTransport::People, 5),
                PlayerObjectiveCard::new("Passengers".to_string(), 17, 20, 28, vec![VehicleType::Electric], TypeEntitiesToTransport::People, 3),
                PlayerObjectiveCard::new("Passengers".to_string(), 27, 27, 15, vec![VehicleType::Electric], TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Packages".to_string(), 23, 24, 7, Vec::new(), TypeEntitiesToTransport::Packages, 5),
            ]
        ),
        SituationCard::new(
            5,
            "Airport train stops".to_string(),
            "No train from City Centre to Airport during rush hours. Delays for passengers.".to_string(),
            "Passengers reach airport in time.".to_string(),
            vec![
                CostTuple::new(Neighbourhood::IndustryPark, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Suburbs, Traffic::LevelTwo),
                CostTuple::new(Neighbourhood::Port, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::RingRoad, Traffic::LevelFour),
                CostTuple::new(Neighbourhood::CityCentre, Traffic::LevelOne),
                CostTuple::new(Neighbourhood::Airport, Traffic::LevelFour),
            ],
            vec![
                PlayerObjectiveCard::new("Passengers".to_string(), 23, 10, 27, vec![VehicleType::Electric], TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Passengers".to_string(), 0, 2, 27, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Passengers".to_string(), 5, 7, 28, Vec::new(), TypeEntitiesToTransport::People, 5),
                PlayerObjectiveCard::new("Passengers".to_string(), 16, 10, 28, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Passengers".to_string(), 14, 10, 27, Vec::new(), TypeEntitiesToTransport::People, 4),
                PlayerObjectiveCard::new("Packages".to_string(), 23, 24, 8, Vec::new(), TypeEntitiesToTransport::Packages, 5),
            ]
        ),
    ]
}
