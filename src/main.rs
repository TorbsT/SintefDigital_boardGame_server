use actix_cors::Cors;
use game_core::{
    game_controller::GameController,
    game_data::{NewGameInfo, Player, PlayerInput, GameState},
};
use serde::{Serialize, Deserialize};
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
    
    let games = game_controller.get_created_games();
    
    games.iter().find(|&g| g.id == *id).map_or_else(||
        HttpResponse::InternalServerError().body(format!("Could not find the game with id {}", id.clone())), 
        |game| HttpResponse::Ok().json(json!(game.clone())))
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

#[get("/games/lobbies")]
async fn get_lobbies(shared_data: web::Data<AppData>) -> impl Responder {
    let Ok(game_controller) = shared_data.game_controller.lock() else {
        return HttpResponse::InternalServerError().body("Failed to get lobbies because the server could not lock the game controller for safe use".to_string());
    };

    let lobbies = LobbyList{ lobbies: game_controller.get_all_lobbies() };
    HttpResponse::Ok().json(json!(lobbies))
}

#[delete("/games/leave/{id}")]
async fn leave_game(id: web::Path<i32>, shared_data: web::Data<AppData>) -> impl Responder {
    let Ok(mut game_controller) = shared_data.game_controller.lock() else { 
        return HttpResponse::InternalServerError().body("Failed to get amount of player IDs because could not lock game controller".to_string());
    };
    game_controller.remove_player_from_game(*id);
    HttpResponse::Ok().body("")
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
                .service(test)
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
        game_controller: Mutex::new(GameController::new(logger.clone())),
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
    use game_core::game_data::{Node, GameState, PlayerInputType};

    fn create_game_controller() ->web::Data<AppData> {
        let logger = Arc::new(RwLock::new(ThresholdLogger::new(
            LogLevel::Ignore,
            LogLevel::Ignore,
        )));
                
        web::Data::new(AppData {
            game_controller: Mutex::new(GameController::new(logger)),
        })
    }

    fn body_to_player_id(data: Bytes) -> i32 {
        String::from_utf8_lossy(&data).trim().parse::<i32>().unwrap()
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
    
    #[actix_web::test]
    async fn test_getting_player_ids() {
        let app_data = create_game_controller();
        
        let app =
            test::init_service(server_app_with_data!(app_data)).await;

        let mut ids: Vec<i32> = Vec::new();

        for _ in 0..5_000 {
            let req = test::TestRequest::get()
                .uri("/create/playerID")
                .to_request();
            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body = test::read_body(resp).await;
            assert!(!body.is_empty());

            let body_str = String::from_utf8_lossy(&body);
            let id = body_str.trim().parse::<i32>().unwrap();

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

        let start_node = Node::new(1, "Start".to_string());
        let end_node = Node::new(2, "End".to_string());

        let mut player = make_player!(app, "P1");
        player.position = Some(start_node.clone());

        let game_state: GameState = make_new_lobby_with_player!(app, player, "Lobby1");

        player = game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.clone().position.unwrap().id == start_node.id);

        let input = PlayerInput {player: player.clone(), input_type: PlayerInputType::Movement, related_node: end_node.clone()};
        let input_req = test::TestRequest::post().uri("/games/input").set_json(&input).to_request();
        let input_resp = app.call(input_req).await.unwrap();
        assert_eq!(input_resp.status(), StatusCode::OK);
        let changed_game_state: GameState = test::read_body_json(input_resp).await;
        
        player = changed_game_state.players.into_iter().find(|p| p.unique_id == player.unique_id).unwrap();
        assert!(player.position.unwrap().id == end_node.id);
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
        let lobby_list: Vec<GameState> = test::read_body_json(lobby_list_resp).await;
        
        lobby_list.iter().for_each(|lobby| {
            assert!(lobby.is_lobby);
            assert_eq!(lobby.players.len(), 1);
            assert!(lobby.players.iter().any(|p| p.unique_id == player1.unique_id || p.unique_id == player2.unique_id || p.unique_id == player3.unique_id)); 
        });

        // TODO: Once the orchestrator can start a game, we need to check if a started game does not return
    }
    
}
