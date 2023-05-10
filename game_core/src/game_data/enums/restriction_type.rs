use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RestrictionType {
    ParkAndRide,
    Electric,
    Emergency,
    Hazard,
    Destination,
    Heavy,
    OneWay, // This should never be chosen as a district restriction
}

impl RestrictionType {
    pub const fn times_to_increase_traffic_when_access(&self) -> usize {
        match self {
            Self::ParkAndRide => 0,
            Self::Electric => 2,
            Self::Emergency => 0,
            Self::Hazard => 1,
            Self::Destination => 1,
            Self::Heavy => 1,
            Self::OneWay => 0, // This should never be chosen as a district restriction
        }
    }
}