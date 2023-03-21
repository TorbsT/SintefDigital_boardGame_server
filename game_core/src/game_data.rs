use serde::{Deserialize, Serialize};

//// =============== Enums ===============
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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u8,
    pub name: String,
    pub neighbours_id: Vec<(u8, NeighbourRelationship)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighbourRelationship {
    pub id: u8,
    pub group: u8,
    pub cost: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeMap {
    pub map: Vec<Node>,
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
        if self.contains_player_with_unique_id(player.unique_id) {
            return Err(
                "A player that is already assigned to a game cannot be assigned again".to_string(),
            );
        }
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
        }
    }
}

impl Node {
    #[must_use]
    pub const fn new(id: u8, name: String) -> Self {
        Self {
            id,
            name,
            neighbours_id: Vec::new(),
        }
    }

    pub fn add_neighbour_id(&mut self, neighbour_id: u8, relationship: NeighbourRelationship) {
        self.neighbours_id.push((neighbour_id, relationship));
    }
}

impl NeighbourRelationship {
    #[must_use]
    pub const fn new(id: u8, group: u8) -> Self {
        Self {
            id,
            group,
            cost: 1,
        }
    }

    pub fn update_cost(&mut self, update: u8) {
        self.cost = update;
    }
}

impl NodeMap {
    #[must_use]
    pub fn new() -> Self {
        let mut map: Vec<Node> = Vec::new();
        map.push(Node::new(0, String::from("Factory")));
        map.push(Node::new(1, String::from("Refinery")));
        map.push(Node::new(2, String::from("Industry Park")));
        map.push(Node::new(3, String::from("I1")));
        map.push(Node::new(4, String::from("I2")));
        map.push(Node::new(5, String::from("Port")));
        map.push(Node::new(6, String::from("I3")));
        map.push(Node::new(7, String::from("Beach")));
        map.push(Node::new(8, String::from("Northside")));
        map.push(Node::new(9, String::from("I4")));
        map.push(Node::new(10, String::from("Central Station")));
        map.push(Node::new(11, String::from("City Square")));
        map.push(Node::new(12, String::from("Concert Hall")));
        map.push(Node::new(13, String::from("Eastside Mart")));
        map.push(Node::new(14, String::from("East Town")));
        map.push(Node::new(15, String::from("Food Court")));
        map.push(Node::new(16, String::from("City Park")));
        map.push(Node::new(17, String::from("Quarry")));
        map.push(Node::new(18, String::from("I5")));
        map.push(Node::new(19, String::from("I6")));
        map.push(Node::new(20, String::from("I7")));
        map.push(Node::new(21, String::from("I8")));
        map.push(Node::new(22, String::from("West Town")));
        map.push(Node::new(23, String::from("Lakeside")));
        map.push(Node::new(24, String::from("Warehouses")));
        map.push(Node::new(25, String::from("I9")));
        map.push(Node::new(26, String::from("I10")));
        map.push(Node::new(27, String::from("Terminal 1")));
        map.push(Node::new(28, String::from("Terminal 2")));
        Self {
            map,
        }
    }
}
