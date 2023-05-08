#[cfg(test)]
pub mod test {
    use std::sync::{RwLock, Mutex, Arc};

    use crate::{AppData, LobbyList, get_unique_id, create_new_game, get_amount_of_created_player_ids, get_gamestate, handle_player_input, get_lobbies, join_game, leave_game, get_situation_cards, player_check_in};
    use actix_web::{dev::Service, http::StatusCode, test, web::{self, Bytes}, App};
    use actix_cors::Cors;
    use game_core::{
        game_data::{custom_types::PlayerID,
            structs::{gamestate::GameState, new_game_info::NewGameInfo, player::Player, node_map::NodeMap, situation_card::SituationCard, cost_tuple::CostTuple, situation_card_list::SituationCardList, player_input::PlayerInput},
            enums::{player_input_type::PlayerInputType, in_game_id::InGameID, district::District, traffic::Traffic}},
            game_controller::GameController, situation_card_list::situation_card_list_wrapper};
    use logging::{threshold_logger::ThresholdLogger, logger::LogLevel};
    use rules::game_rule_checker::GameRuleChecker;

    fn create_game_controller() ->web::Data<AppData> {
        let logger = Arc::new(RwLock::new(ThresholdLogger::new(
            LogLevel::Ignore,
            LogLevel::Ignore,
        )));
                
        web::Data::new(AppData {
            game_controller: Mutex::new(GameController::new(logger, Box::new(GameRuleChecker::new()))),
        })
    }

    fn body_to_player_id(data: Bytes) -> PlayerID {
        String::from_utf8_lossy(&data).trim().parse::<PlayerID>().unwrap()
    }

    macro_rules! server_app_with_data {
        ($x:expr) => {
            {
                let cors = Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials();
        
                App::new()
                    .wrap(cors)
                    .app_data($x.clone())
                    .service(get_unique_id)
                    .service(create_new_game)
                    .service(get_amount_of_created_player_ids)
                    .service(get_gamestate)
                    .service(handle_player_input)
                    .service(get_lobbies)
                    .service(join_game)
                    .service(leave_game)
                    .service(get_situation_cards)
                    .service(player_check_in)
            }
        }
    }

    macro_rules! get_id {
        ($x:expr) => {
            {
                let id_req = test::TestRequest::get().uri("/create/playerID").to_request();
                let id_resp = $x.call(id_req).await.unwrap();
                body_to_player_id(test::read_body(id_resp).await)
            }
        }
    }

    macro_rules! make_player {
        ($app:expr, $player_name:expr) => {
            {
                let id = get_id!($app);
                Player::new(id, $player_name.to_string())
            }
        }
    }

    macro_rules! make_new_lobby_with_player {
        ($app:expr, $player:expr, $lobby_name:expr) => {
            {
                let new_game_info = NewGameInfo {host: $player.clone(), name: $lobby_name.to_string()};
                let create_new_game_req = test::TestRequest::post().uri("/create/game").set_json(&new_game_info).to_request();
                let new_game_resp = $app.call(create_new_game_req).await.unwrap();
                let game: GameState = test::read_body_json(new_game_resp).await;
                game
            }
        }
    }
    
    macro_rules! get_lobbies {
        ($app:expr) => {
            {
                let lobby_list_req = test::TestRequest::get().uri("/games/lobbies").to_request();
                let lobby_list_resp = $app.call(lobby_list_req).await.unwrap();
                let lobby_list: LobbyList = test::read_body_json(lobby_list_resp).await;
                lobby_list.lobbies.clone()
            }
        };
    }

    macro_rules! join_lobby {
        ($app:expr, $game:expr, $player:expr) => {
            {
                let join_game_req = test::TestRequest::post().uri(format!("/games/join/{}", $game.id).as_str()).set_json($player.clone()).to_request();
                let join_game_resp = $app.call(join_game_req).await.unwrap();
                let returned_game: GameState = test::read_body_json(join_game_resp).await;
                returned_game
            }
        }
    }

    macro_rules! leave_lobby {
        ($app:expr, $player:expr) => {
            {
                let player_leave_request = test::TestRequest::delete().uri(format!("/games/leave/{}", $player.unique_id).as_str()).to_request();
                $app.call(player_leave_request).await.unwrap();
            }
        }
    }

    #[actix_web::test]
    async fn test_getting_player_ids() {
        let app_data = create_game_controller();
        
        let app =
            test::init_service(server_app_with_data!(app_data)).await;

        let mut ids: Vec<PlayerID> = Vec::new();

        for _ in 0..5_000 {
            let req = test::TestRequest::get()
                .uri("/create/playerID")
                .to_request();
            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body = test::read_body(resp).await;
            assert!(!body.is_empty());

            let body_str = String::from_utf8_lossy(&body);
            let id = body_str.trim().parse::<PlayerID>().unwrap();

            assert!(ids.iter().all(|i| i != &id));
            ids.push(id);
        }
    }

    #[actix_web::test]
    async fn test_creating_game() {
        let app_data = create_game_controller();
        let app =
            test::init_service(server_app_with_data!(app_data)).await;
   
        let mut player = make_player!(app, "P1");

        let game_name = "Lobby one";
        let new_game_info = NewGameInfo {host: player.clone(), name: game_name.to_string()};
        
        let create_new_game_req = test::TestRequest::post().uri("/create/game").set_json(&new_game_info).to_request();
        let new_game_resp = app.call(create_new_game_req).await.unwrap();
        assert_eq!(new_game_resp.status(), StatusCode::OK);
        let game_state: GameState = test::read_body_json(new_game_resp).await;

        assert!(game_state.players.clone().into_iter().any(|p| p.unique_id == player.unique_id));
        player = game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.connected_game_id.is_some());
        assert!(player.connected_game_id.unwrap() == game_state.id);
        assert!(game_state.name == game_name);

        let player2 = make_player!(app, "P2");
        let game_2_name = "Lobby two";
        let game2: GameState = make_new_lobby_with_player!(app, player2, game_2_name);
        assert!(game2.players.clone().into_iter().any(|p| p.unique_id == player2.unique_id));
        assert!(game2.id != game_state.id);
        assert!(game2.name == game_2_name);
        assert!(game2.name != game_state.name);
    }
    
    #[actix_web::test]
    async fn test_moving_player() {
        let app_data = create_game_controller();
        let app =
            test::init_service(server_app_with_data!(app_data)).await;

        let node_map = NodeMap::new_default();
        let start_node = node_map.nodes.get(0).unwrap();
        let neighbour_info = node_map.edges.get(&0).unwrap().first().unwrap();

        let mut player = make_player!(app, "P1");
        player.position_node_id = Some(start_node.id);
        player.remaining_moves = 1;

        let game_state: GameState = make_new_lobby_with_player!(app, player, "Lobby1");

        player = game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.clone().position_node_id.unwrap() == start_node.id);

        let input = PlayerInput {district_modifier: None, player_id: player.unique_id, game_id: player.connected_game_id.unwrap(), input_type: PlayerInputType::Movement, related_role: None, related_node_id: Some(neighbour_info.to), situation_card_id: None, edge_modifier: None, related_bool: None};

        let input_req = test::TestRequest::post().uri("/games/input").set_json(&input).to_request();
        let input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        // let changed_game_state: GameState = test::read_body_json(input_resp).await;
        
        // player = changed_game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        // assert!(player.position_node_id.unwrap() == neighbour_info.0);
    }

    #[actix_web::test]
    async fn test_getting_lobbies() {
        let app_data = create_game_controller();
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let player1 = make_player!(app, "p1");
        let player2 = make_player!(app, "p2");
        let player3 = make_player!(app, "p3");

        make_new_lobby_with_player!(app, player1, "Lobby1");
        make_new_lobby_with_player!(app, player2, "Lobby2");
        make_new_lobby_with_player!(app, player3, "Lobby3");

        let lobby_list_req = test::TestRequest::get().uri("/games/lobbies").to_request();
        let lobby_list_resp = app.call(lobby_list_req).await.unwrap();
        assert_eq!(lobby_list_resp.status(), StatusCode::OK);
        let lobby_list: LobbyList = test::read_body_json(lobby_list_resp).await;
        
        lobby_list.lobbies.iter().for_each(|lobby| {
            assert!(lobby.is_lobby);
            assert_eq!(lobby.players.len(), 1);
            assert!(lobby.players.iter().any(|p| p.unique_id == player1.unique_id || p.unique_id == player2.unique_id || p.unique_id == player3.unique_id)); 
        });

    }

    // Lag test her
    #[actix_web::test]
    async fn test_player_joining_game() {
        let app_data = create_game_controller();
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let player1 = make_player!(app, "p1");
        let player2 = make_player!(app, "p2");

        let game = make_new_lobby_with_player!(app, player1, "Lobby1");

        let join_game_req = test::TestRequest::post().uri(format!("/games/join/{}", game.id).as_str()).set_json(player2.clone()).to_request();
        let join_game_resp = app.call(join_game_req).await.unwrap();
        assert_eq!(join_game_resp.status(), StatusCode::OK);
        let mut returned_game: GameState = test::read_body_json(join_game_resp).await;
        assert!(returned_game.players.iter().any(|p| p.unique_id == player1.unique_id));
        assert!(returned_game.players.iter().any(|p| p.unique_id == player2.unique_id));

        let lobbies = get_lobbies!(app);
        returned_game = lobbies.into_iter().find(|g| g.id == game.id).unwrap();
        assert!(returned_game.players.iter().any(|p| p.unique_id == player1.unique_id));
        assert!(returned_game.players.iter().any(|p| p.unique_id == player2.unique_id));
    }
    
    #[actix_web::test]
    async fn test_leaving_game() {
        let app_data = create_game_controller();
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let player1 = make_player!(app, "p1");

        let mut lobby = make_new_lobby_with_player!(app, player1, "Lobby1");
        assert!(lobby.players.iter().any(|p| p.unique_id == player1.unique_id));
        
        let player_leave_request = test::TestRequest::delete().uri(format!("/games/leave/{}", player1.unique_id).as_str()).to_request();
        let player_leave_response = app.call(player_leave_request).await.unwrap();
        assert_eq!(player_leave_response.status(), StatusCode::OK);

        let mut lobbies = get_lobbies!(app);
        assert!(lobbies.iter().all(|l| l.players.iter().all(|p| p.unique_id != player1.unique_id)));
        
        lobby = make_new_lobby_with_player!(app, player1, "Lobby2");
        let player2 = make_player!(app, "p2");

        lobby = join_lobby!(app, lobby, player2);
        assert!(lobby.players.iter().any(|p| p.unique_id == player2.unique_id));
        
        leave_lobby!(app, player2);
        lobbies = get_lobbies!(app);
       
        assert!(lobbies.iter().all(|l| l.players.iter().all(|p| p.unique_id != player2.unique_id)));
        assert!(lobbies.iter().any(|l| l.players.iter().any(|p| p.unique_id == player1.unique_id)));
    }

    #[actix_web::test]
    async fn test_changing_role() {
        let app_data = create_game_controller();
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let mut player = make_player!(app, "Player One");    
        let mut lobby = make_new_lobby_with_player!(app, player, "Lobby One");

        assert!(lobby.players.iter().any(|p| p.unique_id == player.unique_id && p.in_game_id == InGameID::Undecided));
        
        player = lobby.players.iter().find(|p| p.unique_id == player.unique_id).unwrap().clone();
        
        let player_input = PlayerInput{district_modifier: None, player_id: player.unique_id, game_id: lobby.id, input_type: PlayerInputType::ChangeRole, related_role: Some(InGameID::Orchestrator), related_node_id: None, situation_card_id: None, edge_modifier: None, related_bool: None};

        let mut input_req = test::TestRequest::post().uri("/games/input").set_json(&player_input).to_request();
        let mut input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::OK);
        lobby = test::read_body_json(input_resp).await;

        assert!(lobby.players.iter().any(|p| p.unique_id == player.unique_id && p.in_game_id == InGameID::Orchestrator));

        let player2 = make_player!(app, "Player Two");
        lobby = join_lobby!(app, lobby, player2);
        
        let player2_input = PlayerInput{district_modifier: None, player_id: player2.unique_id, game_id: lobby.id, input_type: PlayerInputType::ChangeRole, related_role: Some(InGameID::Orchestrator), related_node_id: None, situation_card_id: None, edge_modifier: None, related_bool: None};
        input_req = test::TestRequest::post().uri("/games/input").set_json(&player2_input).to_request();
        input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        let lobbies = get_lobbies!(app);
        lobby = lobbies.into_iter().find(|g| g.id == lobby.id).unwrap();
        assert!(lobby.players.iter().any(|p| p.unique_id == player.unique_id && p.in_game_id == InGameID::Orchestrator));
        assert!(lobby.players.iter().any(|p| p.unique_id == player2.unique_id && p.in_game_id == InGameID::Undecided));
    }

    #[actix_web::test]
    async fn test_if_player_exists() {
        let app_data = create_game_controller();
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let player = make_player!(app, "P1");
        let lobby = make_new_lobby_with_player!(app, player, "L1");
        assert!(player.unique_id != 0);

        let mut player_input = PlayerInput{district_modifier: None, player_id: player.unique_id, game_id: lobby.id, input_type: PlayerInputType::ChangeRole, related_role: Some(InGameID::Orchestrator), related_node_id: None, situation_card_id: None, edge_modifier: None, related_bool: None};
        
        let mut input_req = test::TestRequest::post().uri("/games/input").set_json(&player_input).to_request();
        let mut input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::OK);

        player_input.player_id = 0;
        player_input.related_role = Some(InGameID::PlayerOne);
        
        input_req = test::TestRequest::post().uri("/games/input").set_json(&player_input).to_request();
        input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_get_situation_card() {
        let mut gamestate = GameState::new("Test".to_string(), 42);
        assert!(gamestate.situation_card.is_none());
        let situation_card = SituationCard::new(
            0,
            "Situation Test Scenario".to_string(),
            "Traffic is arbitrarily selected in this scenario".to_string(),
            "Test to see that situation cards work as intended".to_string(),
            vec![
                CostTuple::new(District::IndustryPark, Traffic::LevelOne),
                CostTuple::new(District::Port, Traffic::LevelTwo),
                CostTuple::new(District::Suburbs, Traffic::LevelThree),
                CostTuple::new(District::RingRoad, Traffic::LevelFour),
                CostTuple::new(District::CityCentre, Traffic::LevelFive),
                CostTuple::new(District::Airport, Traffic::LevelThree),
            ],
            Vec::new()
        );
        gamestate.update_situation_card(situation_card);
        assert!(gamestate.situation_card.is_some());
    }
    #[actix_web::test]
    async fn test_get_situation_card_list() {
        let internal_situation_card_list = situation_card_list_wrapper();

        let app_data = create_game_controller();
        
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let situation_card_list_req = test::TestRequest::get().uri("/resources/situationcards").to_request();
        let situation_card_list_resp = app.call(situation_card_list_req).await.unwrap();
        assert_eq!(situation_card_list_resp.status(), StatusCode::OK);
        let situation_card_list: SituationCardList = test::read_body_json(situation_card_list_resp).await;

        assert_eq!(situation_card_list, internal_situation_card_list);

    }
    #[actix_web::test]
    async fn test_assign_situation_card() {
        let app_data = create_game_controller();
        let app = test::init_service(server_app_with_data!(app_data)).await;

        let mut player = make_player!(app, "P1");

        let game_state: GameState = make_new_lobby_with_player!(app, player, "Lobby1");
        player = game_state.players.iter().find(|p| p.unique_id == player.unique_id).unwrap().clone();

        let situation_card_list = situation_card_list_wrapper();
        let situation_card_id = Some(situation_card_list.situation_cards.get(0).cloned().unwrap().card_id);

        let input = PlayerInput {district_modifier: None, player_id: player.unique_id, game_id: player.connected_game_id.unwrap(), input_type: PlayerInputType::AssignSituationCard, related_role: None, related_node_id: None, situation_card_id, edge_modifier: None, related_bool: None};
        let input_req = test::TestRequest::post().uri("/games/input").set_json(&input).to_request();
        let input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::OK);

    }
}
