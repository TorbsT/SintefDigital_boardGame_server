use crate::game_data::{GameState, PlayerInput};

pub trait RuleChecker {
    fn is_input_valid(&self, game: &GameState, input: &PlayerInput) -> Result<bool, String>;
}
