use game_core::{game_controller::GameController, game_data::{Player, InGameID, Node, NewGameInfo, PlayerInput}};
use std::sync::{Arc, RwLock, Mutex};

use actix_web::{get, post, App, Responder, HttpResponse, HttpServer, web};
use logging::{threshold_logger::ThresholdLogger, logger::LogLevel};
use serde_json::json;

struct AppData {
    game_controller: Mutex<GameController>,
}

#[get("/test")]
async fn test() -> impl Responder {
    let p = Player {connected_game_id: 0, in_game_id: InGameID::PlayerOne, unique_id: 0, name: "Player one".to_string(), position: Node {id: 1, name: "One".to_string(), neighbours_id: Vec::new()}};
    let lobby = NewGameInfo {host: p, name: "Lobby one".to_string()};
    HttpResponse::Ok().json(json!(lobby))
}

#[get("/create/playerID")]
async fn get_unique_id(shared_data: web::Data<AppData>) -> impl Responder {
    match shared_data.game_controller.lock().unwrap().generate_player_id() {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to make player ID because: {e}")),
    }
}

#[post("/create/game")]
async fn create_new_game(json_data: web::Json<NewGameInfo>, shared_data: web::Data<AppData>) -> impl Responder {
    let lobby_info = json_data.into_inner();
    match shared_data.game_controller.lock().unwrap().create_new_game(lobby_info) {
        Ok(g) => HttpResponse::Ok().json(json!(g)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to create game because: {e}")),
    }
}

#[get("/debug/playerIDs/amount")]
async fn get_amount_of_created_player_ids(shared_data: web::Data<AppData>) -> impl Responder {
    HttpResponse::Ok().body(shared_data.game_controller.lock().unwrap().get_amount_of_created_player_ids().to_string())
}

#[get("/games/{id}")]
async fn get_gamestate(id: web::Path<i32>, shared_data: web::Data<AppData>) -> impl Responder {
    let games = shared_data.game_controller.lock().unwrap().get_created_games();
    match games.iter().find(|&g| g.id == *id) {
        Some(game) => HttpResponse::Ok().json(json!(game.clone())),
        None => HttpResponse::InternalServerError().body(format!("Could not find the game with id {}", id.clone()))
    }
}

#[post("/games/input")]
async fn handle_player_input(json_data: web::Json<PlayerInput>, shared_data: web::Data<AppData>) -> impl Responder {
    let input = json_data.into_inner();
    match shared_data.game_controller.lock().unwrap().handle_player_input(input) {
        Ok(g) => HttpResponse::Ok().json(json!(g)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to make move because {e}")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let logger = Arc::new(RwLock::new(ThresholdLogger::new(LogLevel::Ignore, LogLevel::Ignore)));
    let app_data = web::Data::new(AppData {game_controller: Mutex::new(GameController::new(logger.clone()))});

    HttpServer::new(move || {
        App::new()
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
    use actix_web::{http::StatusCode, test, dev::Service, App, web};
    
    #[actix_web::test]
    async fn test_getting_player_ids() {

        let logger = Arc::new(RwLock::new(ThresholdLogger::new(LogLevel::Ignore, LogLevel::Ignore)));
        let app_data = web::Data::new(AppData {game_controller: Mutex::new(GameController::new(logger.clone()))});

        // Create App instance
        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .service(get_unique_id),
        ).await;

        let mut ids: Vec<i32> = Vec::new();

        // Make `num_requests` GET requests to /create/playerID
        for _ in 0..50_000 {
            let req = test::TestRequest::get().uri("/create/playerID").to_request();
            let resp = app.call(req).await.unwrap();

            // Check that the response is a 200 OK
            assert_eq!(resp.status(), StatusCode::OK);

            // Check that the response body is not empty
            let body = test::read_body(resp).await;
            assert!(!body.is_empty());

            let body_str = String::from_utf8_lossy(&body);
            let id = body_str.trim().parse::<i32>().unwrap();

            assert!(ids.iter().all(|i| i != &id));
            ids.push(id);
        }
    }
}