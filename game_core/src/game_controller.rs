use std::{
    any::type_name,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use logging::logger::{LogData, LogLevel, Logger};

use crate::{
    game_data::{
        GameID, GameState, NewGameInfo, Player, PlayerID, PlayerInput, PlayerInputType,
        SituationCardList,
    },
    rule_checker::RuleChecker,
};

pub const PLAYER_TIMEOUT: Duration = Duration::from_secs(90);

pub struct GameController {
    pub games: Vec<GameState>,
    pub unique_ids: Vec<(i32, Instant)>,
    pub logger: Arc<RwLock<dyn Logger + Send + Sync>>,
    pub rule_checker: Box<dyn RuleChecker + Send + Sync>,
}

impl GameController {
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

    pub fn get_created_games(&mut self) -> Vec<GameState> {
        self.remove_empty_games();
        self.games.clone()
    }

    pub fn generate_player_id(&mut self) -> Result<i32, &str> {
        let new_id = match self.generate_unused_unique_id() {
            Some(i) => i,
            None => return Err("Failed to make new ID!"),
        };

        self.unique_ids.push((new_id, Instant::now()));

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
        self.remove_empty_games();
        self.remove_inactive_ids();

        if !self
            .unique_ids
            .iter()
            .any(|(id, _)| id == &player_input.player_id)
        {
            return Err("There does not exist a player with the unique id".to_string());
        }

        let mut games_iter = self.games.iter_mut();

        let connected_game_id = player_input.game_id;

        let related_game = match games_iter.find(|game| game.id == connected_game_id) {
            Some(game) => game,
            None => {
                return Err("Could not find the game the player has done an input for!".to_string())
            }
        };

        let mut related_game_clone = related_game.clone();
        match Self::apply_game_actions(&mut related_game_clone) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        if let Some(error) = self
            .rule_checker
            .is_input_valid(&related_game_clone, &player_input)
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
                return Err(e);
            }
        };
        let mut game_clone = related_game.clone();
        match Self::apply_game_actions(&mut game_clone) {
            Ok(_) => Ok(game_clone.clone()),
            Err(e) => Err(e),
        }
        // Ok(related_game.clone())
    }

    pub fn get_amount_of_created_player_ids(&self) -> i32 {
        self.unique_ids.len() as i32
    }

    pub fn get_all_lobbies(&self) -> Vec<GameState> {
        let mut lobbies = Vec::new();
        self.games.clone().into_iter().for_each(|game| {
            if game.is_lobby {
                lobbies.push(game);
            }
        });
        lobbies
    }

    pub fn remove_player_from_game(&mut self, player_id: i32) {
        self.games.iter_mut().for_each(|game| {
            if game.players.iter().any(|p| p.unique_id == player_id) {
                game.remove_player_with_id(player_id);
            }
        });
        self.remove_empty_games();
    }

    pub fn join_game(&mut self, game_id: i32, player: Player) -> Result<GameState, String> {
        for game in self.games.iter() {
            if game.contains_player_with_unique_id(player.unique_id) {
                return Err("The player is already connected to another game.".to_string());
            }
        }
        let mut games_iter = self.games.iter_mut();
        let related_game = match games_iter.find(|game| game.id == game_id) {
            Some(game) => game,
            None => {
                return Err("Could not find the game the player has done an input for!".to_string())
            }
        };
        match related_game.assign_player_to_game(player) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        Ok(related_game.clone())
    }

    pub fn get_game_by_id(&self, game_id: GameID) -> Result<GameState, String> {
        let Some(game) = self.games.iter().find(|g| g.id == game_id) else {
            return Err(format!("There is no game with id {}!", game_id));
        };
        let mut game_clone = game.clone();
        match Self::apply_game_actions(&mut game_clone) {
            Ok(_) => Ok(game_clone.clone()),
            Err(e) => Err(e),
        }
    }

    pub fn update_check_in_and_remove_inactive(
        &mut self,
        player_id: PlayerID,
    ) -> Result<(), String> {
        if self.unique_ids.iter().all(|(id, _)| id != &player_id) {
            return Err(format!("Player with id {} does not exist!", player_id));
        }
        for mut id in self.unique_ids.iter_mut() {
            if id.0 == player_id {
                id.1 = Instant::now();
            }
        }
        self.remove_inactive_ids();
        self.remove_empty_games();
        Ok(())
    }

    fn remove_empty_games(&mut self) {
        self.games.retain(|game| !game.players.is_empty());
    }

    fn remove_inactive_ids(&mut self) {
        self.unique_ids
            .retain(|(_, last_checkin)| last_checkin.elapsed() < PLAYER_TIMEOUT);
        let remaining_ids = self.unique_ids.clone();
        self.games.iter_mut().for_each(|game| {
            game.players
                .retain(|player| remaining_ids.iter().any(|(id, _)| &player.unique_id == id));
        });
    }

    fn change_role_player(input: PlayerInput, game: &mut GameState) -> Result<(), &str> {
        let Some(related_role) = input.related_role else {
            return Err("There was no related role to change to!");
        };
        game.assign_player_role((input.player_id, related_role))
    }

    fn generate_unused_unique_id(&mut self) -> Option<i32> {
        if let Ok(mut logger) = self.logger.write() {
            logger.log(LogData::new(
                LogLevel::Debug,
                "Making new player ID",
                type_name::<Self>(),
            ));
        }

        let mut id: i32 = rand::random::<i32>();

        let mut found_unique_id = false;
        for _ in 0..100_000 {
            {
                if !self.unique_ids.iter().any(|(l_id, _)| l_id == &id) {
                    found_unique_id = true;
                    break;
                }
            }
            id = rand::random::<i32>();
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
        if self
            .unique_ids
            .iter()
            .all(|(id, _)| id != &new_lobby.host.unique_id)
        {
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

    fn generate_unused_game_id(&self) -> i32 {
        let mut existing_game_ids = Vec::new();
        for game in self.games.iter() {
            existing_game_ids.push(game.id);
        }

        let mut id = rand::random::<i32>();
        while existing_game_ids.contains(&id) {
            id = rand::random::<i32>();
        }

        id
    }

    fn apply_game_actions(game: &mut GameState) -> Result<(), String> {
        for action in game.actions.clone().iter() {
            match Self::apply_input(action.clone(), game) {
                Ok(_) => (),
                Err(e) => return Err(e + " No actions are applied to the game."),
            };
        }
        Ok(())
    }

    fn game_next_turn(game: &mut GameState) -> Result<(), String> {
        let mut game_clone = game.clone();
        match Self::apply_game_actions(&mut game_clone) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        std::mem::swap(game, &mut game_clone);
        game.actions.clear();
        game.next_player_turn();
        Ok(())
    }

    fn add_action(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        let mut game_clone = game.clone();
        for action in game.actions.iter() {
            match Self::apply_input(action.clone(), &mut game_clone) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        match Self::apply_input(input.clone(), &mut game_clone) {
            Ok(_) => game.actions.push(input),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn handle_input(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        if input.input_type == PlayerInputType::NextTurn {
            return Self::game_next_turn(game);
        } else if input.input_type == PlayerInputType::UndoAction {
            match game.actions.pop() {
                Some(_) => return Ok(()),
                None => return Err("There is no action to undo!".to_string()),
            }
        } else if input.input_type == PlayerInputType::ChangeRole
            || input.input_type == PlayerInputType::StartGame
            || input.input_type == PlayerInputType::AssignSituationCard
            || input.input_type == PlayerInputType::LeaveGame
        {
            match Self::apply_input(input, game) {
                Ok(_) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        Self::add_action(input, game)
    }

    fn apply_input(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        match input.input_type {
            PlayerInputType::Movement => match Self::handle_movement(input, game) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            PlayerInputType::ChangeRole => match Self::change_role_player(input, game) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            },
            PlayerInputType::All => {
                Err("This input type should not be used by players".to_string())
            }
            PlayerInputType::NextTurn => Err(
                "This is not an action that can be handled by GameController::apply_input!"
                    .to_string(),
            ),
            PlayerInputType::UndoAction => {
                Err("This cannot be done in GameController::apply_input!".to_string())
            }
            PlayerInputType::ModifyDistrict => {
                match Self::handle_district_restriction(input, game) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            PlayerInputType::StartGame => match game.start_game() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            PlayerInputType::AssignSituationCard => {
                let Some(id) = input.situation_card_id else {
                    return Err("There was no situation card id in the input, maybe deserialization problem?".to_string());
                };
                match SituationCardList::get_default_situation_card_by_id(id) {
                    Ok(card) => {
                        game.situation_card = Some(card);
                        match game.update_node_map_with_situation_card() {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            PlayerInputType::LeaveGame => {
                game.remove_player_with_id(input.player_id);
                Ok(())
            }
            PlayerInputType::ModifyEdgeRestrictions => {
                let Some(edge_mod) = input.edge_modifier else {
                    return Err("There was no park and ride modifier when wanting to modify park and ride!".to_string());
                };
                if edge_mod.delete {
                    return game.remove_restriction_from_edge(&edge_mod);
                }
                game.add_edge_restriction(&edge_mod)
            }
            PlayerInputType::SetPlayerTrainBool => {
                let Some(boolean) = input.related_bool else {
                    return Err("There was no bool to set the train bool with!".to_string());
                };
                game.set_player_train_bool(input.player_id, boolean);
                Ok(())
            },
            PlayerInputType::SetPlayerBusBool => {
                let Some(boolean) = input.related_bool else {
                    return Err("There was no bool to set the bus bool with!".to_string());
                };
                game.set_player_bus_bool(input.player_id, boolean);
                Ok(())
            },
        }
    }

    fn handle_movement(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        let Some(related_node_id) = input.related_node_id else {
            return Err("There was no node related to the movement!".to_string());
        };
        match game.move_player_with_id(input.player_id, related_node_id) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to move player because: {e}")),
        }

        match game.update_objective_status() {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn handle_district_restriction(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        let Some(district_modifier) = input.district_modifier else {
            return Err("There was no district in the input modifier even though it was marked as a district input".to_string());
        };
        if district_modifier.delete {
            return game.remove_district_modifier(district_modifier);
        }
        game.add_district_modifier(district_modifier)
    }
}
