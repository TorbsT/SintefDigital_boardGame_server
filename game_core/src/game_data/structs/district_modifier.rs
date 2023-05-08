use serde::{Deserialize, Serialize};

use crate::game_data::{enums::{district::District, district_modifier_type::DistrictModifierType, restriction_type::RestrictionType}, custom_types::{MovementValue, Money}};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DistrictModifier {
    pub district: District,
    pub modifier: DistrictModifierType,
    pub vehicle_type: Option<RestrictionType>,
    pub associated_movement_value: Option<MovementValue>,
    pub associated_money_value: Option<Money>,
    pub delete: bool,
}