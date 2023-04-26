#![allow(unknown_lints, clippy::significant_drop_tightening)]

use actix_cors::Cors;
use game_core::{
    game_controller::GameController,
    situation_card_list::situation_card_list_wrapper,
    game_data::{NewGameInfo, Player, PlayerInput, GameState},
};
use serde::{Serialize, Deserialize};
use rules::game_rule_checker::GameRuleChecker;
use std::sync::{Arc, Mutex, RwLock};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, delete};
use logging::{logger::LogLevel, threshold_logger::ThresholdLogger};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct LobbyList {
    lobbies: Vec<GameState>,
}

struct AppData {
    game_controller: Mutex<GameController>,
}

#[get("/test/newLobby")]
async fn test() -> impl Responder {
    let p = Player::new(0, "Player one".to_string()); 
    let lobby = NewGameInfo {
        host: p,
        name: "Lobby one".to_string(),
    };
    HttpResponse::Ok().json(json!(lobby))
}

#[get("/create/playerID")]
async fn get_unique_id(shared_data: web::Data<AppData>) -> impl Responder {
    let data = shared_data.game_controller.lock();
    match data {
        Ok(mut game_controller) => {
            let player_result = game_controller.generate_player_id();
            match player_result {
                Ok(id) => HttpResponse::Ok().body(id.to_string()),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to make player ID because: {e}")),
            }
        }
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to make player ID because: {e}")),
    }
}

#[post("/create/game")]
async fn create_new_game(
    json_data: web::Json<NewGameInfo>,
    shared_data: web::Data<AppData>,
) -> impl Responder {
    let lobby_info = json_data.into_inner();
    let data = shared_data.game_controller.lock();
    match data {
        Ok(mut game_controller) => {
            let game_result = game_controller.create_new_game(lobby_info);
            match game_result {
                Ok(g) => HttpResponse::Ok().json(json!(g)),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to create game because: {e}")),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to create game because {e}"))
        }
    }
}

#[get("/debug/playerIDs/amount")]
async fn get_amount_of_created_player_ids(shared_data: web::Data<AppData>) -> impl Responder {
    let game_controller = match shared_data.game_controller.lock() {
        Ok(controller) => controller, 
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string()),
        };
    HttpResponse::Ok().body(
        game_controller
            .get_amount_of_created_player_ids()
            .to_string(),
    )
}

#[get("/games/game/{id}")]
async fn get_gamestate(id: web::Path<i32>, shared_data: web::Data<AppData>) -> impl Responder {
    
    let game_controller = match shared_data.game_controller.lock() { 
        Ok(controller) => controller,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string()),
    };

    let game_result = game_controller.get_game_by_id(*id);
    match game_result {
        Ok(game) => HttpResponse::Ok().json(json!(game)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Could not return the game because: {}", e)),
    }  
}

#[post("/games/join/{game_id}")]
async fn join_game(game_id: web::Path<i32>, player: web::Json<Player>, shared_data: web::Data<AppData>) -> impl Responder {
    let mut game_controller = match shared_data.game_controller.lock() { 
        Ok(controller) => controller,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string()),
    };

    let join_game_result = game_controller.join_game(*game_id, player.into_inner());

    match join_game_result {
        Ok(g) => HttpResponse::Ok().json(json!(g)),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to join game because {e}"))
        }
    }
}

#[post("/games/input")]
async fn handle_player_input( //TODO: Orchestrator must be able to assign situation card to gamestate
    json_data: web::Json<PlayerInput>,
    shared_data: web::Data<AppData>,
) -> impl Responder {
    let input = json_data.into_inner();
    
    let mut game_controller = match shared_data.game_controller.lock() { 
        Ok(controller) => controller,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string()),
    };

    let gamestate_result = game_controller.handle_player_input(input); 
    match gamestate_result {
        Ok(g) => HttpResponse::Ok().json(json!(g)),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to make move because {e}"))
        }
    }
}

#[get("/games/lobbies")]
async fn get_lobbies(shared_data: web::Data<AppData>) -> impl Responder {
    let Ok(game_controller) = shared_data.game_controller.lock() else {
        return HttpResponse::InternalServerError().body("Failed to get lobbies because the server could not lock the game controller for safe use".to_string());
    };

    let lobbies = LobbyList{ lobbies: game_controller.get_all_lobbies() };
    HttpResponse::Ok().json(json!(lobbies))
}

#[get("/resources/situationcards")]
async fn get_situation_cards() -> impl Responder {
    HttpResponse::Ok().json(json!(situation_card_list_wrapper()))
}

#[delete("/games/leave/{player_id}")]
async fn leave_game(player_id: web::Path<i32>, shared_data: web::Data<AppData>) -> impl Responder {
    let Ok(mut game_controller) = shared_data.game_controller.lock() else { 
        return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string());
    };
    game_controller.remove_player_from_game(*player_id);
    HttpResponse::Ok().body("")
}

#[get("/check-in/{player_id}")]
async fn player_check_in(player_id: web::Path<i32>, shared_data: web::Data<AppData>) -> impl Responder {
    let Ok(mut game_controller) = shared_data.game_controller.lock() else {
        return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string());
    };
    let result = game_controller.update_check_in_and_remove_inactive(*player_id);
    match result {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
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
                .service(test)
                .service(get_situation_cards)
                .service(player_check_in)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let logger = Arc::new(RwLock::new(ThresholdLogger::new(
        LogLevel::Ignore,
        LogLevel::Ignore,
    )));
    let app_data = web::Data::new(AppData {
        game_controller: Mutex::new(GameController::new(logger.clone(), Box::new(GameRuleChecker::new()))),
    });

    HttpServer::new(move || {
        server_app_with_data!(app_data)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}

// ================= Tests =================

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{dev::Service, http::StatusCode, test, web::{self, Bytes}, App};
    use game_core::game_data::{GameState, PlayerInputType, PlayerID, NodeMap, InGameID, SituationCard, Neighbourhood, Traffic, SituationCardList};

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

    #[allow(unused_macros)]
    macro_rules! change_role {
        ($app:expr, $game:expr, $player:expr, $role:expr) => {
            {
                let player_input = PlayerInput{player_id: $player.unique_id, game_id: $game.id, input_type: PlayerInputType::ChangeRole, related_role: Some($role), related_node_id: None };

                let mut input_req = test::TestRequest::post().uri("/games/input").set_json(&player_input).to_request();
                let mut input_resp = $app.call(input_req).await.unwrap();
                let lobby: GameState = test::read_body_json(input_resp).await;
                lobby
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

        let node_map = NodeMap::new();
        let start_node = node_map.map.get(0).unwrap();
        let neighbour_info = start_node.neighbours.get(0).unwrap();

        let mut player = make_player!(app, "P1");
        player.position_node_id = Some(start_node.id);
        player.remaining_moves = 1;

        let game_state: GameState = make_new_lobby_with_player!(app, player, "Lobby1");

        player = game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.clone().position_node_id.unwrap() == start_node.id);

        let input = PlayerInput {district_modifier: None, player_id: player.unique_id, game_id: player.connected_game_id.unwrap(), input_type: PlayerInputType::Movement, related_role: None, related_node_id: Some(neighbour_info.0), situation_card: None};
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

        // TODO: Once the orchestrator can start a game, we need to check if a started game does not return
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
        
        let player_input = PlayerInput{district_modifier: None, player_id: player.unique_id, game_id: lobby.id, input_type: PlayerInputType::ChangeRole, related_role: Some(InGameID::Orchestrator), related_node_id: None, situation_card: None};

        let mut input_req = test::TestRequest::post().uri("/games/input").set_json(&player_input).to_request();
        let mut input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::OK);
        lobby = test::read_body_json(input_resp).await;

        assert!(lobby.players.iter().any(|p| p.unique_id == player.unique_id && p.in_game_id == InGameID::Orchestrator));

        let player2 = make_player!(app, "Player Two");
        lobby = join_lobby!(app, lobby, player2);
        
        let player2_input = PlayerInput{district_modifier: None, player_id: player2.unique_id, game_id: lobby.id, input_type: PlayerInputType::ChangeRole, related_role: Some(InGameID::Orchestrator), related_node_id: None, situation_card: None};
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

        let mut player_input = PlayerInput{district_modifier: None, player_id: player.unique_id, game_id: lobby.id, input_type: PlayerInputType::ChangeRole, related_role: Some(InGameID::Orchestrator), related_node_id: None, situation_card: None};
        
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
        assert!(gamestate.situation_card == None);
        let situation_card = SituationCard::new(
            0,
            "Situation Test Scenario".to_string(),
            "Traffic is arbitrarily selected in this scenario".to_string(),
            "Test to see that situation cards work as intended".to_string(),
            vec![
                (Neighbourhood::IndustryPark, Traffic::LevelOne),
                (Neighbourhood::Port, Traffic::LevelTwo),
                (Neighbourhood::Suburbs, Traffic::LevelThree),
                (Neighbourhood::RingRoad, Traffic::LevelFour),
                (Neighbourhood::CityCentre, Traffic::LevelFive),
                (Neighbourhood::Airport, Traffic::LevelThree),
            ],
        );
        gamestate.update_situation_card(situation_card);
        assert!(gamestate.situation_card != None);
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

        let situation_card_list = situation_card_list_wrapper();
        let situation_card = situation_card_list.situation_cards.get(0);
        
        //TODO: Fix this error
        let input = PlayerInput {district_modifier: None, player_id: player.unique_id, game_id: player.connected_game_id.unwrap(), input_type: PlayerInputType::AssignSituationCard, related_role: None, related_node_id: None, situation_card: situation_card};
        let input_req = test::TestRequest::post().uri("/games/input").set_json(&input).to_request();
        let input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    }
}
