use std::{
    any::type_name,
    sync::{Arc, RwLock},
    time::{Instant},
};

use logging::logger::{LogData, LogLevel, Logger};

use crate::{
    rule_checker::RuleChecker, game_data::{structs::{gamestate::GameState, new_game_info::NewGameInfo, player_input::PlayerInput, player::Player, situation_card_list::SituationCardList}, custom_types::{GameID, PlayerID, NodeID}, enums::{player_input_type::PlayerInputType}, constants::PLAYER_TIMEOUT},
};

/// The GameController struct is the game manager and is what should be used to control all of the games on the server. It has all the neccessary functions to create and handle games.
pub struct GameController {
    pub games: Vec<GameState>,
    pub unique_ids: Vec<(i32, Instant)>,
    pub logger: Arc<RwLock<dyn Logger + Send + Sync>>,
    pub rule_checker: Box<dyn RuleChecker + Send + Sync>,
}

macro_rules! log {
    ($logger:expr, $level:expr, $message:expr) => {
        if let Ok(mut logger) = $logger.write() {
            logger.log(LogData::new($level, $message, type_name::<Self>()));
        }
    };
}

impl GameController {
    /// Creates a new game and assigns the host to the game.
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

    /// Gets all the created games on the server.
    pub fn get_created_games(&mut self) -> Vec<GameState> {
        self.remove_empty_games();
        self.games.clone()
    }

    /// Generates a new unique id that a player can use and returns it, but also puts it in the list of unique ids that the controller has.
    pub fn generate_player_id(&mut self) -> Result<PlayerID, &str> {
        log!(self.logger, LogLevel::Debug, "Generating new player ID");
        let new_id = match self.generate_unused_unique_id() {
            Some(i) => i,
            None => {
                log!(self.logger, LogLevel::Error, "Failed to make new ID!");
                return Err("Failed to make new ID!")
            },
        };

        self.unique_ids.push((new_id, Instant::now()));

        log!(self.logger, LogLevel::Debug, format!("Made unique ID: {}", new_id).as_str());
        
        Ok(new_id)
    }

    /// Creates a new game based and assigns the host (the one who requested to create a game) to the game.
    pub fn create_new_game(&mut self, new_lobby: NewGameInfo) -> Result<GameState, String> {
        let new_game = match self.create_new_game_and_assign_host(new_lobby) {
            Ok(game) => game,
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to create new game because: {}", e).as_str());
                return Err(e)
            },
        };
        log!(self.logger, LogLevel::Info, format!("Created new game with id: {}", new_game.id).as_str());
        self.games.push(new_game.clone());
        Ok(new_game)
    }

    /// Handles the player input and returns the new game state if the player input was valid.
    pub fn handle_player_input(&mut self, player_input: PlayerInput) -> Result<GameState, String> {
        log!(self.logger, LogLevel::Debug, format!("Handling player input: {:?}", player_input).as_str());
        self.remove_empty_games();
        self.remove_inactive_ids();

        if !self
            .unique_ids
            .iter()
            .any(|(id, _)| id == &player_input.player_id)
        {
            log!(self.logger, LogLevel::Error, format!("There does not exist a player with the unique id {} and can therefore not handle the player input", player_input.player_id).as_str());
            return Err("There does not exist a player with the unique id".to_string());
        }

        let mut games_iter = self.games.iter_mut();

        let connected_game_id = player_input.game_id;

        let related_game = match games_iter.find(|game| game.id == connected_game_id) {
            Some(game) => game,
            None => {
                log!(self.logger, LogLevel::Error, "Could not find the game the player has done an input for!");
                return Err("Could not find the game the player has done an input for!".to_string())
            }
        };
        log!(self.logger, LogLevel::Debug, format!("Found game with id: {}", related_game.id).as_str());

        let mut related_game_clone = related_game.clone();
        match Self::apply_game_actions(&mut related_game_clone) {
            Ok(_) => (),
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to apply previous game actions to the clone of the game with id: {} because: {}", related_game.id, e).as_str());
                return Err(e);
            },
        }
        log!(self.logger, LogLevel::Debug, format!("Applied previous game actions to the clone of the game with id: {}", related_game.id).as_str());

        if let Some(error) = self
            .rule_checker
            .is_input_valid(&related_game_clone, &player_input)
        {
            log!(self.logger, LogLevel::Error, format!("The input was not valid for the game with id: {} because: {}", related_game.id, error).as_str());
            return Err(format!("The input was not valid! Because: {error}"));
        }
        log!(self.logger, LogLevel::Debug, format!("The input was valid for the game with id: {}", related_game.id).as_str());

        match Self::handle_input(player_input.clone(), related_game) {
            Ok(_) => (),
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to handle player input because: {}", e).as_str());
                return Err(e);
            }
        };
        log!(self.logger, LogLevel::Info, format!("Added/Handled the new input to the game with id: {}", related_game.id).as_str());

        let mut game_clone = related_game.clone();
        match Self::apply_game_actions(&mut game_clone) {
            Ok(_) => {
                self.get_legal_nodes(&mut game_clone, player_input.player_id);
                Ok(game_clone.clone())
            },
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to apply the game actions to the clone of the game with id: {} because: {}", related_game.id, e).as_str());
                Err(e)
            },
        }
    }

    /// Returns the amount of unique player ids that have been created.
    pub fn get_amount_of_created_player_ids(&self) -> i32 {
        self.unique_ids.len() as i32
    }

    /// Returns all the games that have not started yet.
    pub fn get_all_lobbies(&self) -> Vec<GameState> {
        log!(self.logger, LogLevel::Debug, "Getting all lobbies!");
        let mut lobbies = Vec::new();
        self.games.clone().into_iter().for_each(|game| {
            if game.is_lobby {
                lobbies.push(game);
            }
        });
        lobbies
    }

    /// Removes/Disconnects the player with the given unique id from the game the player is connected to. This function will also remove all games that do not have any players in them.
    pub fn remove_player_from_game(&mut self, player_id: PlayerID) {
        log!(self.logger, LogLevel::Info, format!("Removing player with id: {}", player_id).as_str());
        self.games.iter_mut().for_each(|game| {
            if game.players.iter().any(|p| p.unique_id == player_id) {
                game.remove_player_with_id(player_id);
            }
        });
        self.remove_empty_games();
    }

    /// Adds the player to the game if there is room for the player and the player is not in another game. It will also return other errors if it cannot add the player to the game.
    pub fn join_game(&mut self, game_id: GameID, player: Player) -> Result<GameState, String> {
        log!(self.logger, LogLevel::Debug, format!("Player with id: {} is trying to join game with id: {}", player.unique_id, game_id).as_str());
        for game in self.games.iter() {
            if game.contains_player_with_unique_id(player.unique_id) {
                log!(self.logger, LogLevel::Error, format!("The player with id: {} is already connected to another game.", player.unique_id).as_str());
                return Err("The player is already connected to another game.".to_string());
            }
        }
        let mut games_iter = self.games.iter_mut();
        let related_game = match games_iter.find(|game| game.id == game_id) {
            Some(game) => game,
            None => {
                log!(self.logger, LogLevel::Error, format!("Could not find the game the player with id: {} is trying to join!", player.unique_id).as_str());
                return Err("Could not find the game the player is trying to join!".to_string())
            }
        };
        match related_game.assign_player_to_game(player.clone()) {
            Ok(_) => (),
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to assign player with id: {} to game with id: {} because: {}", player.unique_id, game_id, e).as_str());
                return Err(e);
            },
        };
        log!(self.logger, LogLevel::Info, format!("Player with id: {} joined game with id: {}", player.unique_id, game_id).as_str());
        Ok(related_game.clone())
    }

    /// Gets the game with the given id. If there was a problem with getting the game it will return a string with the error.
    pub fn get_game_by_id(&mut self, game_id: GameID) -> Result<GameState, String> {
        log!(self.logger, LogLevel::Debug, format!("Trying to get game with id: {}", game_id).as_str());
        let Some(game) = self.games.iter().find(|g| g.id == game_id) else {
            log!(self.logger, LogLevel::Error, format!("There is no game with id {} and can therefore not return the wanted game!", game_id).as_str());
            return Err(format!("There is no game with id {}!", game_id));
        };
        let mut game_clone = game.clone();
        match Self::apply_game_actions(&mut game_clone) {
            Ok(_) => {
                if !game_clone.is_lobby {
                    let current_players_turn = game_clone.current_players_turn;
                    let players = game_clone.players.clone();
                    let Some(player) = players.iter().find(|p| p.in_game_id == current_players_turn) else {
                        log!(self.logger, LogLevel::Error, format!("Failed to apply the game actions to the clone of the game with id {} because there is no player that has the current in game turn {:?} and can therefore not return the wanted game!", game_id, current_players_turn).as_str());
                        return Err(format!("There is no player that has the current in game turn {:?}!", current_players_turn));
                    };
                    self.get_legal_nodes(&mut game_clone, player.unique_id);
                }
                log!(self.logger, LogLevel::Info, format!("Returning game with id: {}", game_id).as_str());
                Ok(game_clone.clone())},
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to apply the game actions to the clone of the game with id: {} because: {} and can therefore not return the wanted game", game_id, e).as_str());
                Err(e)
            },
        }
    }

    /// Tells the game controller that a unique id is used by a player. This will also remove all inactive players. This means that if a player has not checked in after [`PLAYER_TIMEOUT`], they will be removed.
    /// 
    /// [`PLAYER_TIMEOUT`]: const.PLAYER_TIMEOUT.html
    pub fn update_check_in_and_remove_inactive(
        &mut self,
        player_id: PlayerID,
    ) -> Result<(), String> {
        log!(self.logger, LogLevel::Debug, format!("Updating check in for player with id: {}", player_id).as_str());
        if self.unique_ids.iter().all(|(id, _)| id != &player_id) {
            log!(self.logger, LogLevel::Error, format!("Player with id {} does not exist and can therefore not update the check in!", player_id).as_str());
            return Err(format!("Player with id {} does not exist!", player_id));
        }
        for mut id in self.unique_ids.iter_mut() {
            if id.0 == player_id {
                id.1 = Instant::now();
            }
        }
        self.remove_inactive_ids();
        self.remove_empty_games();
        log!(self.logger, LogLevel::Debug, format!("Updated check in for player with id {} and removed unused ids and empty games!", player_id).as_str());
        Ok(())
    }

    fn remove_empty_games(&mut self) {
        log!(self.logger, LogLevel::Debug, "Removing empty games!");
        self.games.retain(|game| !game.players.is_empty());
    }

    fn remove_inactive_ids(&mut self) {
        log!(self.logger, LogLevel::Debug, "Removing inactive ids!");
        self.unique_ids
            .retain(|(_, last_checkin)| last_checkin.elapsed() < PLAYER_TIMEOUT);
        let remaining_ids = self.unique_ids.clone();
        self.games.iter_mut().for_each(|game| {
            game.players
                .retain(|player| remaining_ids.iter().any(|(id, _)| &player.unique_id == id));
        });
        log!(self.logger, LogLevel::Debug, "Removed inactive ids!");
    }

    fn change_role_player(input: PlayerInput, game: &mut GameState) -> Result<(), &str> {
        let Some(related_role) = input.related_role else {
            return Err("There was no related role to change to!");
        };
        game.assign_player_role((input.player_id, related_role))
    }

    fn generate_unused_unique_id(&mut self) -> Option<PlayerID> {
        log!(self.logger, LogLevel::Debug, "Generating unused unique id!");
        let mut id: PlayerID = rand::random::<PlayerID>();
        let mut found_unique_id = false;
        for _ in 0..100_000 {
            {
                if !self.unique_ids.iter().any(|(l_id, _)| l_id == &id) {
                    found_unique_id = true;
                    break;
                }
            }
            id = rand::random::<PlayerID>();
        }

        if !found_unique_id {
            return None;
        }

        log!(self.logger, LogLevel::Debug, format!("Generated unused unique id: {}", id).as_str());
        Some(id)
    }

    fn create_new_game_and_assign_host(
        &mut self,
        new_lobby: NewGameInfo,
    ) -> Result<GameState, String> {
        log!(self.logger, LogLevel::Debug, format!("Trying to create a new game with name {} and assigning host with id {}", new_lobby.name, new_lobby.host.unique_id).as_str());
        if self
            .unique_ids
            .iter()
            .all(|(id, _)| id != &new_lobby.host.unique_id)
        {
            log!(self.logger, LogLevel::Error, "A player that has a unique ID that was not made by the server cannot create a lobby and can therefore not create a new game");
            return Err("A player that has a unique ID that was not made by the server cannot create a lobby.".to_string());
        }

        for game in self.games.iter() {
            if game.contains_player_with_unique_id(new_lobby.host.unique_id) {
                log!(self.logger, LogLevel::Error, "A player that is already connected to a game in progress cannot create a new game");
                return Err("A player that is already connected to a game in progress cannot create a new game.".to_string());
            }
        }

        let mut new_game = GameState::new(new_lobby.name.clone(), self.generate_unused_game_id());
        match new_game.assign_player_to_game(new_lobby.host.clone()) {
            Ok(_) => (),
            Err(e) => {
                log!(self.logger, LogLevel::Error, format!("Failed to assign host with id {} to the new game because: {}", new_lobby.host.unique_id, e).as_str());
                return Err(format!("Failed to create new game because: {e}"));
            },
        };
        log!(self.logger, LogLevel::Info, format!("Created new game with name {} and assigned host with id {}", new_lobby.name, new_lobby.host.unique_id).as_str());
        Ok(new_game)
    }

    fn generate_unused_game_id(&self) -> GameID {
        log!(self.logger, LogLevel::Debug, "Trying to generate unused game id!");
        let mut existing_game_ids = Vec::new();
        for game in self.games.iter() {
            existing_game_ids.push(game.id);
        }

        let mut id = rand::random::<GameID>();
        while existing_game_ids.contains(&id) {
            id = rand::random::<GameID>();
        }
        log!(self.logger, LogLevel::Debug, format!("Generated unused game id: {}", id).as_str());
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
                game.add_edge_restriction(&edge_mod, true)
            }
            PlayerInputType::SetPlayerBusBool => {
                let Some(boolean) = input.related_bool else {
                    return Err("There was no bool to set the bus bool with!".to_string());
                };
                game.set_player_bus_bool(input.player_id, boolean);
                Ok(())
            },
        }
    }

    fn get_legal_nodes(&mut self, game: &mut GameState, player_id: PlayerID) {
        log!(self.logger, LogLevel::Debug, format!("Getting legal nodes for player with id {}!", player_id).as_str());
        let mut legal_nodes: Vec<NodeID> = Vec::new();

        let player =  match game.get_player_with_unique_id(player_id) {
            Ok(player) => player,
            Err(_) => {
                return;
            },
        };

        let Some(current_player_node_id) = player.position_node_id else {
            return;
        };

        let neighbouring_node_relationships = match game.map.get_neighbour_relationships_of_node_with_id(current_player_node_id) {
            Some(neighbours) => neighbours,
            None => {
                return;
            },
        };

        let Some(connected_game_id) = player.connected_game_id else {
            return;
        };

        for relationship in neighbouring_node_relationships {
            let input = PlayerInput {
                district_modifier: None, 
                player_id: player.unique_id, 
                game_id: connected_game_id, 
                input_type: PlayerInputType::Movement, 
                related_role: None, 
                related_node_id: Some(relationship.to), 
                situation_card_id: None, 
                edge_modifier: None, 
                related_bool: None
            };
            self.rule_checker.is_input_valid(game, &input).map_or_else(|| legal_nodes.push(relationship.to), |e| log!(self.logger, LogLevel::Debug, format!("Input was not valid because: {}", e).as_str()));
        }
        game.legal_nodes = legal_nodes;
        log!(self.logger, LogLevel::Debug, format!("Got legal nodes for player with id {}!", player_id).as_str());
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
