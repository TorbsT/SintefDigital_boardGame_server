use actix_cors::Cors;
use game_core::{
    game_controller::GameController,
    game_data::{NewGameInfo, Player, PlayerInput, GameID, LobbyInfo, GameState, GameStartInput},
};
use rules::game_rule_checker::GameRuleChecker;
use std::sync::{Arc, Mutex, RwLock};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use logging::{logger::LogLevel, threshold_logger::ThresholdLogger};
use serde_json::json;

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

/*TODO: Check if orchestrator can start game; start game
        Game can only be started if the lobby has an orchestrator and at least 1 more player
*/
#[post("/start/game")]
async fn start_new_game(
    json_data: web::Json<GameStartInput>,
    shared_data: web::Data<AppData>,
) -> impl Responder {
    let game_start_input: GameStartInput = json_data.into_inner();
    let data = shared_data.game_controller.lock();
    match data {
        Ok(mut game_controller) => {
            let games = game_controller.get_created_games();
            let mut gamestate: GameState = GameState::new("null".to_owned(), 0);
            for game in games {
                if game.id == game_start_input.game_id {
                    gamestate = game;
                    break;
                }
            }
            if gamestate.name == "null" && gamestate.id == 0 {
                return HttpResponse::InternalServerError().body("Failed to start game because: Failed to find game");
            }
            let game_result = game_controller.start_game(&mut gamestate);
            match game_result {
                Ok(g) => HttpResponse::Ok().json(json!(g)),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to start game because: {e}")),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to start game because {e}"))
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

#[get("/games/{id}")]
async fn get_gamestate(id: web::Path<GameID>, shared_data: web::Data<AppData>) -> impl Responder {
    
    let game_controller = match shared_data.game_controller.lock() { 
        Ok(controller) => controller,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string()),
    };
    
    let games = game_controller.get_created_games();
    
    games.iter().find(|&g| g.id == *id).map_or_else(||
        HttpResponse::InternalServerError().body(format!("Could not find the game with id {}", id.clone())), 
        |game| HttpResponse::Ok().json(json!(game.clone())))
}

#[post("/games/input")]
async fn handle_player_input(
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
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(app_data.clone())
            .service(get_unique_id)
            .service(create_new_game)
            .service(get_amount_of_created_player_ids)
            .service(get_gamestate)
            .service(handle_player_input)
            .service(test)
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
    use game_core::game_data::{GameState, PlayerInputType, PlayerID, NodeMap};

    fn create_game_controller() ->web::Data<AppData> {
        let logger = Arc::new(RwLock::new(ThresholdLogger::new(
            LogLevel::Debug,
            LogLevel::Ignore,
        )));
                
        web::Data::new(AppData {
            game_controller: Mutex::new(GameController::new(logger, Box::new(GameRuleChecker::new()))),
        })
    }

    fn body_to_player_id(data: Bytes) -> PlayerID {
        String::from_utf8_lossy(&data).trim().parse::<PlayerID>().unwrap()
    }    
    
    #[actix_web::test]
    async fn test_getting_player_ids() {
        let app_data = create_game_controller();
        
        let app =
            test::init_service(App::new().app_data(app_data.clone()).service(get_unique_id)).await;

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
            test::init_service(App::new().app_data(app_data.clone()).service(get_unique_id).service(create_new_game).service(handle_player_input)).await;

        let id_req = test::TestRequest::get().uri("/create/playerID").to_request();
        let id_resp = app.call(id_req).await.unwrap();
        let id = body_to_player_id(test::read_body(id_resp).await);
        
        let mut player = Player::new(id, "P1".to_string());

        let new_game_info = NewGameInfo {host: player.clone(), name: "Lobby one".to_string()};
        
        let create_new_game_req = test::TestRequest::post().uri("/create/game").set_json(&new_game_info).to_request();
        let new_game_resp = app.call(create_new_game_req).await.unwrap();
        assert_eq!(new_game_resp.status(), StatusCode::OK);
        let game_state: GameState = test::read_body_json(new_game_resp).await;

        assert!(game_state.players.clone().into_iter().any(|p| p.unique_id == player.unique_id));
        player = game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.connected_game_id.is_some());
        assert!(player.connected_game_id.unwrap() == game_state.id);       
    }
    
    #[actix_web::test]
    async fn test_moving_player() {
        let app_data = create_game_controller();
        let app =
            test::init_service(App::new().app_data(app_data.clone()).service(get_unique_id).service(create_new_game).service(handle_player_input)).await;

        let node_map = NodeMap::new();
        let start_node = node_map.map.get(0).unwrap();
        let neighbour_info = start_node.neighbours.get(0).unwrap();

        let id_req = test::TestRequest::get().uri("/create/playerID").to_request();
        let id_resp = app.call(id_req).await.unwrap();
        let id = body_to_player_id(test::read_body(id_resp).await);
        
        let mut player = Player::new(id, "P1".to_string());
        player.position_node_id = Some(start_node.id);
        player.remaining_moves = 1;

        let new_game_info = NewGameInfo {host: player.clone(), name: "Lobby one".to_string()};
        
        let create_new_game_req = test::TestRequest::post().uri("/create/game").set_json(&new_game_info).to_request();
        let new_game_resp = app.call(create_new_game_req).await.unwrap();
        assert_eq!(new_game_resp.status(), StatusCode::OK);
        let game_state: GameState = test::read_body_json(new_game_resp).await;

        player = game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.clone().position_node_id.unwrap() == start_node.id);

        let input = PlayerInput {player_id: player.unique_id, game_id: player.connected_game_id.unwrap(), input_type: PlayerInputType::Movement, related_node_id: Some(neighbour_info.0), related_role: None};
        let input_req = test::TestRequest::post().uri("/games/input").set_json(&input).to_request();
        let input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::OK);
        let changed_game_state: GameState = test::read_body_json(input_resp).await;
        
        player = changed_game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.position_node_id.unwrap() == neighbour_info.0);
    }
}
