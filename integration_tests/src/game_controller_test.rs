#[cfg(test)]
mod tests {
    use std::{sync::{Arc, RwLock}, cmp::min};

    use game_core::{game_data::{PlayerInput, self, Player, NewGameInfo, InGameID, GameState}, rule_checker, game_controller::GameController};
    use game_data::{Node, PlayerInputType, NeighbourRelationship}; 
    use logging::{threshold_logger::ThresholdLogger, logger::LogLevel};
    use rules::game_rule_checker::GameRuleChecker;

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
            remaining_moves: 0,
        };
        player
    }
    
    #[allow(clippy::unwrap_used)]
    fn get_unique_player_id(controller: &mut GameController) -> Result<i32, ()> {
        controller.generate_player_id().map_or(Err(()), Ok)
    }

    fn make_game_controller() -> GameController {
        let logger = Arc::new(RwLock::new(ThresholdLogger::new(
            LogLevel::Ignore,
            LogLevel::Ignore,
        )));
        let rule_checker = Box::new(GameRuleChecker::new());
        GameController::new(logger, rule_checker)
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

    // =============== Tests ===============

    #[test]
    fn test_generation_of_unique_player_ids() {
        assert!(get_all_wanted_unique_player_ids(0));
        assert!(get_all_wanted_unique_player_ids(1));
        assert!(get_all_wanted_unique_player_ids(5));
        assert!(get_all_wanted_unique_player_ids(50));
        assert!(get_all_wanted_unique_player_ids(500));
        assert!(get_all_wanted_unique_player_ids(5000));
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
            input_type: PlayerInputType::Movement,
            related_node: end_node.clone(),
            player_id: player.unique_id,
            game_id: game.id,
        };

        game = controller.handle_player_input(input).expect("Expected to get GameState after doing an input. Seems like something went wrong when handling the input");

        assert!(game.players.iter().any(|p| p.unique_id == player.unique_id));
        assert!(game
            .players
            .iter()
            .any(|p| p.clone().position.unwrap().id == end_node.id));
    }
}
