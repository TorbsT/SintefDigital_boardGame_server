use std::{
    any::type_name,
    sync::{Arc, RwLock},
};

use logging::logger::{LogData, LogLevel, Logger};

use crate::{
    game_data::{self, GameState, NewGameInfo, PlayerInput, PlayerID},
    rule_checker::RuleChecker,
};

// TODO: Jobbet fra 04:00 til 09:00
// TODO: Jobbet fra 13:00 til

pub struct GameController {
    pub games: Vec<GameState>,
    pub unique_ids: Vec<PlayerID>,
    pub logger: Arc<RwLock<dyn Logger + Send + Sync>>,
    pub rule_checker: Box<dyn RuleChecker + Send + Sync>,
}

impl GameController {
    //TODO: Make sure that a player cannot change how many remaining moves they have

    pub fn new(
        logger: Arc<RwLock<dyn Logger + Send + Sync>>,
        rule_checker: Box<dyn RuleChecker + Send + Sync>,
    ) -> Self {
        Self {
            games: Vec::new(),
            unique_ids: Vec::new(),
            logger,
            rule_checker,
        }
    }

    pub fn get_created_games(&self) -> Vec<GameState> {
        self.games.clone()
    }

    pub fn generate_player_id(&mut self) -> Result<PlayerID, &str> {
        let new_id = match self.generate_unused_unique_id() {
            Some(i) => i,
            None => return Err("Failed to make new ID!"),
        };

        self.unique_ids.push(new_id);

        if let Ok(mut logger) = self.logger.write() {
            logger.log(LogData::new(
                LogLevel::Debug,
                "Made unique ID",
                type_name::<Self>(),
            ));
        }
        Ok(new_id)
    }

    pub fn create_new_game(&mut self, new_lobby: NewGameInfo) -> Result<GameState, String> {
        let new_game = match self.create_new_game_and_assign_host(new_lobby) {
            Ok(game) => game,
            Err(e) => return Err(e),
        };

        self.games.push(new_game.clone());
        Ok(new_game)
    }

    pub fn handle_player_input(&mut self, player_input: PlayerInput) -> Result<GameState, String> {
        let mut games_iter = self.games.iter_mut();
        let connected_game_id = player_input.game_id;
        let related_game = match games_iter.find(|game| game.id == connected_game_id) {
            Some(game) => game,
            None => return Err("Could not find the game the player has done an input for!".to_string()),
        };

        if let Some(error) = self
            .rule_checker
            .is_input_valid(related_game, &player_input) 
        {
            if let Ok(mut logger) = self.logger.write() {
                logger.log(LogData::new(
                    LogLevel::Info,
                    error.as_str(),
                    type_name::<Self>(),
                ));
            }
            return Err(format!("The input was not valid! Because: {error}"));
        }

        match Self::handle_input(player_input, related_game) {
            Ok(_) => (),
            Err(e) => {
                if let Ok(mut logger) = self.logger.write() {
                    logger.log(LogData {
                        severity_level: LogLevel::Error,
                        log_data: format!("Failed to handle player input because: {}", e).as_str(),
                        caller_identifier: type_name::<Self>(),
                    })
                }
            }
        };

        Ok(related_game.clone())
    }

    pub fn get_amount_of_created_player_ids(&self) -> i32 {
        self.unique_ids.len() as i32
    }

    fn generate_unused_unique_id(&mut self) -> Option<PlayerID> {
        if let Ok(mut logger) = self.logger.write() {
            logger.log(LogData::new(
                LogLevel::Debug,
                "Making new player ID",
                type_name::<Self>(),
            ));
        }

        let mut id: PlayerID = rand::random::<PlayerID>();

        let mut found_unique_id = false;
        for _ in 0..100_000 {
            {
                if !self.unique_ids.contains(&id) {
                    found_unique_id = true;
                    break;
                }
            }
            id = rand::random::<PlayerID>();
        }

        if !found_unique_id {
            return None;
        }

        if let Ok(mut logger) = self.logger.write() {
            logger.log(LogData::new(
                LogLevel::Debug,
                "Done making new player ID",
                type_name::<Self>(),
            ));
        }

        Some(id)
    }

    fn create_new_game_and_assign_host(
        &mut self,
        new_lobby: NewGameInfo,
    ) -> Result<GameState, String> {
        if !self.unique_ids.contains(&new_lobby.host.unique_id) {
            return Err("A player that has a unique ID that was not made by the server cannot create a lobby.".to_string());
        }

        for game in self.games.iter() {
            if game.contains_player_with_unique_id(new_lobby.host.unique_id) {
                return Err("A player that is already connected to a game in progress cannot create a new game.".to_string());
            }
        }

        let mut new_game = GameState::new(new_lobby.name, self.generate_unused_game_id());
        match new_game.assign_player_to_game(new_lobby.host) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to create new game because: {e}")),
        };
        Ok(new_game)
    }

    fn generate_unused_game_id(&self) -> PlayerID {
        let mut existing_game_ids = Vec::new();
        for game in self.games.iter() {
            existing_game_ids.push(game.id);
        }

        let mut id = rand::random::<PlayerID>();
        while existing_game_ids.contains(&id) {
            id = rand::random::<PlayerID>();
        }

        id
    }

    fn handle_input(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        match input.input_type {
            game_data::PlayerInputType::Movement => match Self::handle_movement(input, game) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
        }
    }

    fn handle_movement(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        match game.move_player_with_id(input.player_id, input.related_node_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to move player because: {e}")),
        }
    }
}