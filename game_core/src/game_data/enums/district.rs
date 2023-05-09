use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum District {
    IndustryPark,
    Port,
    Suburbs,
    RingRoad,
    CityCentre,
    Airport,
}

impl District {
    pub const fn first() -> Self {
        Self::IndustryPark
    }

    pub const fn next(&self) -> Option<Self> {
        match self {
            Self::IndustryPark => Some(Self::Port),
            Self::Port => Some(Self::Suburbs),
            Self::Suburbs => Some(Self::RingRoad),
            Self::RingRoad => Some(Self::CityCentre),
            Self::CityCentre => Some(Self::Airport),
            Self::Airport => None,
        }
    }
}