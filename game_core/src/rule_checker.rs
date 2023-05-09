use crate::game_data::{structs::{player_input::PlayerInput, gamestate::GameState}, custom_types::ErrorData};

/// A trait that defines the interface for a rule checker used by the [`GameController`].
/// 
/// [`GameController`]: struct.GameController.html
pub trait RuleChecker {
    fn is_input_valid(&self, game: &GameState, input: &PlayerInput) -> Option<ErrorData>;
}
