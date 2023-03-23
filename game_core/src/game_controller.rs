use std::{
    any::type_name,
    cmp::min,
    sync::{Arc, RwLock},
};

use logging::logger::{LogData, LogLevel, Logger};

use crate::game_data::{self, GameState, NewGameInfo, Player, PlayerInput, InGameID};

// TODO: Jobbet fra 04:00 til 09:00
// TODO: Jobbet fra 13:00 til

pub struct GameController {
    pub games: Vec<GameState>,
    pub unique_ids: Vec<i32>,
    pub logger: Arc<RwLock<dyn Logger + Send + Sync>>,
}

impl GameController {
    pub fn new(logger: Arc<RwLock<dyn Logger + Send + Sync>>) -> Self {
        Self {
            games: Vec::new(),
            unique_ids: Vec::new(),
            logger,
        }
    }

    pub fn get_created_games(&self) -> Vec<GameState> {
        self.games.clone()
    }

    pub fn generate_player_id(&mut self) -> Result<i32, &str> {
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

    pub fn handle_player_input(&mut self, player_input: PlayerInput) -> Result<GameState, &str> {
        let mut games_iter = self.games.iter_mut();
        let Some(connected_game_id) = player_input.player.connected_game_id else {
            return Err("Player is not connected to a game");
        };

        let related_game = match games_iter.find(|game| game.id == connected_game_id) {
            Some(game) => game,
            None => return Err("Could not find the game the player has done an input for!"),
        };

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
                if !self.unique_ids.contains(&id) {
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

    fn handle_input(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        match input.input_type {
            game_data::PlayerInputType::Movement => match Self::handle_movement(input, game) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
        }
    }

    fn handle_movement(input: PlayerInput, game: &mut GameState) -> Result<(), String> {
        // TODO: Check here if the movement is valid once applicable
        let mut player = input.player;
        player.position = Some(input.related_node);
        match game.update_player(player) {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed to move player because: {e}")),
        }
    }
}

// =========== For testing ===========
use logging::threshold_logger::ThresholdLogger;

fn make_game_controller() -> GameController {
    let logger = Arc::new(RwLock::new(ThresholdLogger::new(
        LogLevel::Ignore,
        LogLevel::Ignore,
    )));
    GameController::new(logger)
}

pub fn get_all_wanted_unique_player_ids(amount_of_new_ids_to_create: usize) -> bool {
    let mut controller = make_game_controller();

    let mut ids = Vec::with_capacity(amount_of_new_ids_to_create);
    for _ in 0..amount_of_new_ids_to_create {
        match controller.generate_player_id() {
            Ok(id) => {
                if ids.contains(&id) {
                    return false;
                }

                ids.push(id);
            }
            Err(_) => return false,
        }
    }

    true
}

#[allow(clippy::unwrap_used)]
pub fn test_creating_new_games(
    amount_of_new_players_to_create: i32,
    amount_of_new_games: i32,
) -> bool {
    let mut controller = make_game_controller();

    let random_players: Vec<Player> =
        make_random_player_list_with_size(amount_of_new_players_to_create, &mut controller);

    let new_lobbies: Vec<NewGameInfo> =
        make_random_new_lobbies(amount_of_new_games, random_players);

    let mut games_created: Vec<GameState> = Vec::new();

    for new_lobby in &new_lobbies {
        if let Ok(game) = controller.create_new_game(new_lobby.clone()) {
            games_created.push(game)
        }
    }

    if games_created.len()
        != min(
            amount_of_new_players_to_create as usize,
            amount_of_new_games as usize,
        )
    {
        return false;
    }

    let mut actual_games_to_create_from_full_list: Vec<NewGameInfo> = Vec::new();
    for i in 0..new_lobbies.len() {
        if actual_games_to_create_from_full_list
            .iter()
            .any(|lobby| lobby.host.unique_id == new_lobbies.get(i).unwrap().host.unique_id)
        {
            continue;
        }
        actual_games_to_create_from_full_list.push(new_lobbies.get(i).unwrap().clone());
    }

    for lobby in actual_games_to_create_from_full_list {
        if !games_created.iter().any(|game| {
            game.players
                .iter()
                .any(|player| player.unique_id == lobby.host.unique_id)
                && game.name == lobby.name
        }) {
            return false;
        }
    }
    true
}

// ========= Helpers ===========
#[allow(clippy::unwrap_used)]
fn make_random_new_lobbies(
    amount_of_new_games: i32,
    random_players: Vec<Player>,
) -> Vec<NewGameInfo> {
    let mut new_lobbies: Vec<NewGameInfo> = Vec::with_capacity(amount_of_new_games as usize);
    let mut player_index = 0;
    for _ in 0..amount_of_new_games {
        if random_players.is_empty() {
            break;
        }
        let player = random_players.get(player_index).unwrap();
        player_index += 1;
        if player_index == random_players.len() {
            player_index = 0;
        }
        new_lobbies.push(NewGameInfo {
            host: player.clone(),
            name: rand::random::<i32>().to_string(),
        });
    }

    new_lobbies
}

#[allow(clippy::unwrap_used)]
fn make_random_player_list_with_size(
    amount_of_new_players_to_create: i32,
    controller: &mut GameController,
) -> Vec<Player> {
    let mut players: Vec<Player> = Vec::with_capacity(amount_of_new_players_to_create as usize);
    for _ in 0..amount_of_new_players_to_create {
        let mut p: Player = make_random_player_info(controller);
        while players.iter().any(|p1| p1.unique_id == p.unique_id) {
            p = make_random_player_info(controller);
        }
        players.push(p);
    }

    players
}

#[allow(clippy::unwrap_used)]
fn make_random_player_info(controller: &mut GameController) -> Player {
    let player: Player = Player {
        connected_game_id: None,
        in_game_id: InGameID::Undecided,
        unique_id: get_unique_player_id(controller).unwrap(),
        name: rand::random::<i32>().to_string(),
        position: None,
    };
    player
}

#[allow(clippy::unwrap_used)]
fn get_unique_player_id(controller: &mut GameController) -> Result<i32, ()> {
    controller.generate_player_id().map_or(Err(()), Ok)
}

//#[cfg(not(test))]
#[cfg(test)]
mod tests {

    use crate::game_data::{Node, PlayerInputType, NeighbourRelationship};

    use super::*;

    #[test]
    fn test_generation_of_unique_player_ids() {
        assert!(get_all_wanted_unique_player_ids(0));
        assert!(get_all_wanted_unique_player_ids(1));
        assert!(get_all_wanted_unique_player_ids(5));
        assert!(get_all_wanted_unique_player_ids(50));
        assert!(get_all_wanted_unique_player_ids(500));
        assert!(get_all_wanted_unique_player_ids(5000));
        assert!(get_all_wanted_unique_player_ids(50000));
    }

    // Here instead of using multiple function calls use #[parameterized(...)]
    #[test]
    fn test_making_lobbies() {
        assert!(test_creating_new_games(0, 0));
        assert!(test_creating_new_games(0, 1));
        assert!(test_creating_new_games(1, 1));
        assert!(test_creating_new_games(5, 10));
        assert!(test_creating_new_games(100, 110));
        assert!(test_creating_new_games(1000, 1000));
    }

    #[test]
    fn test_player_movement() {
        let mut controller = make_game_controller();

        let mut start_node = Node {
            id: 1,
            name: "Start".to_string(),
            neighbours: Vec::new(),
        };
        let mut end_node = Node {
            id: 2,
            name: "End".to_string(),
            neighbours: Vec::new(),
        };
        start_node.add_neighbour(&mut end_node, Arc::new(NeighbourRelationship::new(0, game_data::Neighbourhood::IndustryPark)));

        let mut player = make_random_player_info(&mut controller);
        player.position = Some(start_node);
        let lobby = NewGameInfo {
            host: player.clone(),
            name: "Test".to_string(),
        };

        let mut game = controller.create_new_game(lobby).expect("Expected to get GameState but did not get it. Seems like the game failed to be created.");

        assert!(game.players.iter().any(|p| p.unique_id == player.unique_id
            && p.clone().position.unwrap().id == player.clone().position.unwrap().id));

        player = game
            .players
            .iter()
            .find(|&p| p.unique_id == player.unique_id)
            .unwrap()
            .clone();

        let input = PlayerInput {
            player: player.clone(),
            input_type: PlayerInputType::Movement,
            related_node: end_node.clone(),
        };

        game = controller.handle_player_input(input).expect("Expected to get GameState after doing an input. Seems like something went wrong when handling the input");

        assert!(game.players.iter().any(|p| p.unique_id == player.unique_id));
        assert!(game
            .players
            .iter()
            .any(|p| p.clone().position.unwrap().id == end_node.id));
    }
}
