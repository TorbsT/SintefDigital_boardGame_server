use crate::game_data::{NewGameInfo, GameState, PlayerInput};


pub trait MultiplayerController {
    fn get_wanted_amount_of_unique_ids(&mut self) -> i32;
    fn handle_generated_unique_ids(&mut self, new_ids: Vec<i32>);
    fn get_requested_game_lobbies(&mut self) -> &mut Vec<NewGameInfo>;
    fn send_new_game_state_to_players(&mut self, game_state: GameState);
    fn get_player_inputs_for_game(&mut self, game_id: i32) -> &mut Vec<PlayerInput>;
}