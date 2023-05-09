#![allow(unknown_lints, clippy::significant_drop_tightening)]

use actix_cors::Cors;
use game_core::{game_controller::GameController, game_data::structs::{new_game_info::NewGameInfo, player::Player, player_input::PlayerInput, gamestate::GameState}, situation_card_list::situation_card_list_wrapper};
use serde::{Serialize, Deserialize};
use rules::game_rule_checker::GameRuleChecker;
use std::sync::{Arc, Mutex, RwLock};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, delete};
use logging::{logger::LogLevel, threshold_logger::ThresholdLogger};
use serde_json::json;

mod test;

#[derive(Serialize, Deserialize)]
struct LobbyList {
    lobbies: Vec<GameState>,
}

struct AppData {
    game_controller: Mutex<GameController>,
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
    
    let mut game_controller = match shared_data.game_controller.lock() { 
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
        Ok(g) => {
            HttpResponse::Ok().json(json!(g))
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to do action because: {e}"))
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
