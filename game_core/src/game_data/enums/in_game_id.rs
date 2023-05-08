use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum InGameID {
    Undecided = 0,
    PlayerOne = 1,
    PlayerTwo = 2,
    PlayerThree = 3,
    PlayerFour = 4,
    PlayerFive = 5,
    PlayerSix = 6,
    Orchestrator = 7,
}

impl InGameID {
    pub const fn next(&self) -> Self {
        match self {
            Self::Undecided => Self::Orchestrator,
            Self::PlayerOne => Self::PlayerTwo,
            Self::PlayerTwo => Self::PlayerThree,
            Self::PlayerThree => Self::PlayerFour,
            Self::PlayerFour => Self::PlayerFive,
            Self::PlayerFive => Self::PlayerSix,
            Self::PlayerSix => Self::Orchestrator,
            Self::Orchestrator => Self::PlayerOne,
        }
    }
}