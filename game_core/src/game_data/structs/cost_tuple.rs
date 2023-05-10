use serde::{Deserialize, Serialize};

use crate::game_data::enums::{district::District, traffic::Traffic};

/// The CostTuple struct describes the Traffic in a District.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CostTuple {
    pub neighbourhood: District,
    pub traffic: Traffic,
}

impl CostTuple {
    pub const fn new(neighbourhood: District, traffic: Traffic) -> Self {
        Self {
            neighbourhood,
            traffic,
        }
    }
}