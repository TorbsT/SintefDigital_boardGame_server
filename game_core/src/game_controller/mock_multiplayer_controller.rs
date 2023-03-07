use std::{sync::{Arc, RwLock}, any::type_name};

use logging::logger::{Logger, LogData, LogLevel};

use crate::{game_data::{GameState, NewGameInfo, PlayerInput, self}, multiplayer_controller::MultiplayerController};

use super::GameController;


pub struct MockMultiplayerController {
    game_controller: GameController,
    known_game_states: Vec<GameState>,
    player_inputs: Vec<PlayerInput>,
    unique_ids: Vec<i32>,
    logger: Arc<RwLock<dyn Logger + Send + Sync>>,
}

// impl MultiplayerController for MockMultiplayerController {
//     fn get_wanted_amount_of_unique_ids(&mut self) -> i32 {
//         let amount = self.wanted_amount_of_new_player_ids;
//         self.wanted_amount_of_new_player_ids = 0; 
//         amount
//     }

//     fn handle_generated_unique_ids(&mut self, mut new_ids: Vec<i32>) {
//         self.logger.write().unwrap().Log(LogData::new(LogLevel::Debug, format!("Handling new IDs, adding: {:?}", new_ids).as_str(), type_name::<Self>()));
//         self.unique_ids.append(&mut new_ids)
//     }

//     fn get_requested_game_lobbies(&mut self) -> &mut Vec<NewLobbyInfo> {
//         &mut self.new_lobbies
//     }

//     fn send_new_game_state_to_players(&mut self, game_state: GameState) {
//         self.created_game_states.retain(|game| game.id != game_state.id);
//         self.created_game_states.push(game_state);
//     }

//     fn get_player_inputs_for_game(&mut self, game_id: i32) -> &mut Vec<PlayerInput> {
//         &mut self.player_inputs
//     }
// }

impl MockMultiplayerController {
    pub fn new(game_controller: GameController, logger: Arc<RwLock<dyn Logger + Send + Sync>>) -> MockMultiplayerController {
        MockMultiplayerController {game_controller: game_controller, known_game_states: Vec::new(), player_inputs: Vec::new(), unique_ids: Vec::new(), logger: logger }
    }

    pub fn make_lobbies(&mut self, mut new_lobbies: Vec<NewGameInfo>) {
        for lobby in new_lobbies {
            match self.game_controller.create_new_game(lobby) {
                Ok(game) => self.known_game_states.push(game),
                Err(e) => self.logger.write().unwrap().Log(LogData::new(LogLevel::Error, format!("Failed to create lobby because: {}", e).as_str(), type_name::<Self>())),
            };
        }
    }

    pub fn get_created_games(&self) -> &Vec<GameState> {
        &self.known_game_states
    }

    pub fn handle_player_input(&mut self, player_input: PlayerInput) {
        let game = match self.game_controller.handle_player_input(player_input) {
            Ok(game) => game,
            Err(e) => {
                self.logger.write().unwrap().Log(LogData::new(LogLevel::Error, format!("Failed to create lobby because: {}", e).as_str(), type_name::<Self>()));
                return;
            },
        };
        self.update_game_state(game);
    }

    pub fn generate_id(&mut self) {
        self.unique_ids.push(self.game_controller.generate_player_id().unwrap());
        self.logger.write().unwrap().Log(LogData::new(LogLevel::Debug, "Made new ID", type_name::<Self>()));
    }

    pub fn get_unique_id(&mut self) -> Option<i32> {
        self.logger.write().unwrap().Log(LogData::new(LogLevel::Debug, "Trying to give new ID", type_name::<Self>()));
        if self.unique_ids.is_empty() {
            self.logger.write().unwrap().Log(LogData::new(LogLevel::Warning, "No new IDs gotten", type_name::<Self>()));
            return None;
        }
        self.logger.write().unwrap().Log(LogData::new(LogLevel::Debug, "New ID returning", type_name::<Self>()));
        self.unique_ids.pop()
    }

    fn update_game_state(&mut self, updated_game: GameState) {
        self.known_game_states.iter_mut().for_each(|game| {
            if game.id != updated_game.id {
                return;
            }
            game.update_game(updated_game.clone());
        });
    }
}