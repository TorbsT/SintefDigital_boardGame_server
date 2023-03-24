use serde::{Deserialize, Serialize};

//// =============== Enums ===============
const MAX_PLAYER_COUNT: usize = 6; // TODO: UPDATE THIS IF INGAMEID IS UPDATED
#[derive(Clone, Serialize, Deserialize)]
pub enum InGameID {
    Undecided = 0,
    PlayerOne = 1,
    PlayerTwo = 2,
    PlayerThree = 3,
    PlayerFour = 4,
    PlayerFive = 5,
    Orchestrator = 6,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PlayerInputType {
    Movement,
}

//// =============== Structs ===============
#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: i32,
    pub name: String,
    pub players: Vec<Player>,
    pub is_lobby: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub connected_game_id: Option<i32>,
    pub in_game_id: InGameID,
    pub unique_id: i32,
    pub name: String,
    pub position: Option<Node>,
    pub remaining_moves: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i32,
    pub name: String,
    pub neighbours_id: Vec<i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewGameInfo {
    pub host: Player,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub player: Player,
    pub input_type: PlayerInputType,
    pub related_node: Node,
}

//// =============== Structs impls ===============
impl GameState {
    #[must_use]
    pub const fn new(name: String, game_id: i32) -> Self {
        Self {
            id: game_id,
            name,
            players: Vec::new(),
            is_lobby: true,
        }
    }

    pub fn contains_player_with_unique_id(&self, unique_id: i32) -> bool {
        for player in &self.players {
            if player.unique_id == unique_id {
                return true;
            }
        }
        false
    }

    pub fn assign_player_to_game(&mut self, mut player: Player) -> Result<(), String> {
        if self.players.len() >= MAX_PLAYER_COUNT {
            return Err("The game is full".to_string());
        }

        if self.contains_player_with_unique_id(player.unique_id) {
            return Err(
                "A player that is already assigned to a game cannot be assigned again".to_string(),
            );
        }

        player.in_game_id = InGameID::Undecided;
        player.connected_game_id = Some(self.id);
        self.players.push(player);
        Ok(())
    }

    pub fn update_player(&mut self, player_to_update: Player) -> Result<(), String> {
        for player in self.players.iter_mut() {
            if player.unique_id != player_to_update.unique_id {
                continue;
            }
            player.position = player_to_update.position;
            // TODO: Add the ability to change role in the game aswell when applicable
            return Ok(());
        }
        Err("There were no players in this game that match the player to update".to_string())
    }

    pub fn update_game(&mut self, update: Self) {
        self.players = update.players;
    }
}

impl Player {
    #[must_use]
    pub const fn new(unique_id: i32, name: String) -> Self {
        Self {
            connected_game_id: None,
            in_game_id: InGameID::Undecided,
            unique_id,
            name,
            position: None,
            remaining_moves: 0,
        }
    }
}

impl Node {
    #[must_use]
    pub const fn new(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            neighbours_id: Vec::new(),
        }
    }

    pub fn add_neighbour_id(&mut self, neighbour_id: i32) {
        self.neighbours_id.push(neighbour_id);
    }
}
