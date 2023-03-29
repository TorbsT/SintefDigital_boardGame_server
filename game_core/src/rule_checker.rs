use crate::game_data::{GameState, PlayerInput};

pub type ErrorData = String;

pub trait RuleChecker {
    fn is_input_valid(&self, game: &GameState, input: &PlayerInput) -> Option<ErrorData>;
}
