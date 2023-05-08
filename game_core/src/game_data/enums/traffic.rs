use serde::{Deserialize, Serialize};

use crate::game_data::custom_types::MovementCost;
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Traffic {
    LevelOne,
    LevelTwo,
    LevelThree,
    LevelFour,
    LevelFive,
}

impl Traffic {
    pub const fn get_movement_cost(&self) -> MovementCost {
        match self {
            Self::LevelOne => 0,
            Self::LevelTwo => 0,
            Self::LevelThree => 1,
            Self::LevelFour => 2,
            Self::LevelFive => 4,
        }
    }

    pub const fn increased(&self) -> Self {
        match self {
            Self::LevelOne => Self::LevelTwo,
            Self::LevelTwo => Self::LevelThree,
            Self::LevelThree => Self::LevelFour,
            Self::LevelFour => Self::LevelFive,
            Self::LevelFive => Self::LevelFive,
        }
    }
}