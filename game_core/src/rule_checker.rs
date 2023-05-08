use crate::game_data::structs::{player_input::PlayerInput, gamestate::GameState};



pub type ErrorData = String;

pub trait RuleChecker {
    fn is_input_valid(&self, game: &GameState, input: &PlayerInput) -> Option<ErrorData>;
}
