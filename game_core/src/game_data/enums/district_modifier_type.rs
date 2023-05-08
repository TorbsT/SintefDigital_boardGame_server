use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DistrictModifierType {
    Access,
    Priority,
    Toll,
}