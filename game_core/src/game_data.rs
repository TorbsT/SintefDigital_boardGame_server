use std::sync::{Arc, Mutex};

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
    #[serde(skip)]
    pub neighbours: Vec<(u8, Arc<NeighbourRelationship>)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighbourRelationship {
    pub id: u8,
    pub group: u8,
    pub individual_cost: u8,
    pub total_cost: u8,
}

#[derive(Clone)]
pub struct NodeMap {
    pub map: Vec<Arc<Mutex<Node>>>,
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
            neighbours: Vec::new(),
        }
    }

    pub fn add_neighbour(&mut self, neighbour: &mut Node, relationship: Arc<NeighbourRelationship>) {
        self.neighbours.push((neighbour.id, relationship.clone()));
        neighbour.neighbours.push((self.id, relationship));
    }
}

pub enum Neighbourhood {
    IndustryPark,
    Port,
    Suburbs,
    RingRoad,
    CityCentre,
    Airport,
}

const GROUP_COST_ARRAY: [u8; 6] = [1, 1, 1, 1, 1, 1];

impl NeighbourRelationship {
    #[must_use]
    pub const fn new(id: u8, neighbourhood: Neighbourhood) -> Self {
        let group: u8 = GROUP_COST_ARRAY[neighbourhood as usize];
        Self {
            id,
            group,
            individual_cost: 0,
            total_cost: 1,
        }
    }

    pub fn update_individual_cost(&mut self, update: u8) {
        self.individual_cost = update;
        self.update_total_cost();
    }

    pub fn update_total_cost(&mut self) {
        self.total_cost = self.group + self.individual_cost;
    }
}

impl NodeMap {
    pub fn add_to_map(map: &mut Vec<Arc<Mutex<Node>>>, node: Node) {
        map.push(Arc::new(Mutex::new(node)));
    }

    pub fn add_relationship(node1: &mut Node, node2: &mut Node, relationship: NeighbourRelationship) {
        node1.add_neighbour(node2, Arc::new(relationship));
    }
    #[must_use]
    pub fn new() -> Self {
        let mut map: Vec<Arc<Mutex<Node>>> = Vec::new();
        let mut node0: Node = Node::new(0, String::from("Factory"));
        let mut node1: Node = Node::new(1, String::from("Refinery"));
        let mut node2: Node = Node::new(2, String::from("Industry Park"));
        let mut node3: Node = Node::new(3, String::from("I1"));
        let mut node4: Node = Node::new(4, String::from("I2"));
        let mut node5: Node = Node::new(5, String::from("Port"));
        let mut node6: Node = Node::new(6, String::from("I3"));
        let mut node7: Node = Node::new(7, String::from("Beach"));
        let mut node8: Node = Node::new(8, String::from("Northside"));
        let mut node9: Node = Node::new(9, String::from("I4"));
        let mut node10: Node = Node::new(10, String::from("Central Station"));
        let mut node11: Node = Node::new(11, String::from("City Square"));
        let mut node12: Node = Node::new(12, String::from("Concert Hall"));
        let mut node13: Node = Node::new(13, String::from("Eastside Mart"));
        let mut node14: Node = Node::new(14, String::from("East Town"));
        let mut node15: Node = Node::new(15, String::from("Food Court"));
        let mut node16: Node = Node::new(16, String::from("City Park"));
        let mut node17: Node = Node::new(17, String::from("Quarry"));
        let mut node18: Node = Node::new(18, String::from("I5"));
        let mut node19: Node = Node::new(19, String::from("I6"));
        let mut node20: Node = Node::new(20, String::from("I7"));
        let mut node21: Node = Node::new(21, String::from("I8"));
        let mut node22: Node = Node::new(22, String::from("West Town"));
        let mut node23: Node = Node::new(23, String::from("Lakeside"));
        let mut node24: Node = Node::new(24, String::from("Warehouses"));
        let mut node25: Node = Node::new(25, String::from("I9"));
        let mut node26: Node = Node::new(26, String::from("I10"));
        let mut node27: Node = Node::new(27, String::from("Terminal 1"));
        let mut node28: Node = Node::new(28, String::from("Terminal 2"));
        Self::add_relationship(&mut node0, &mut node1, NeighbourRelationship::new(0, Neighbourhood::IndustryPark));
        Self::add_to_map(&mut map, node0);
        Self::add_to_map(&mut map, node1);
        Self::add_to_map(&mut map, node2);
        Self::add_to_map(&mut map, node3);
        Self::add_to_map(&mut map, node4);
        Self::add_to_map(&mut map, node5);
        Self::add_to_map(&mut map, node6);
        Self::add_to_map(&mut map, node7);
        Self::add_to_map(&mut map, node8);
        Self::add_to_map(&mut map, node9);
        Self::add_to_map(&mut map, node10);
        Self::add_to_map(&mut map, node11);
        Self::add_to_map(&mut map, node12);
        Self::add_to_map(&mut map, node13);
        Self::add_to_map(&mut map, node14);
        Self::add_to_map(&mut map, node15);
        Self::add_to_map(&mut map, node16);
        Self::add_to_map(&mut map, node17);
        Self::add_to_map(&mut map, node18);
        Self::add_to_map(&mut map, node19);
        Self::add_to_map(&mut map, node20);
        Self::add_to_map(&mut map, node21);
        Self::add_to_map(&mut map, node22);
        Self::add_to_map(&mut map, node23);
        Self::add_to_map(&mut map, node24);
        Self::add_to_map(&mut map, node25);
        Self::add_to_map(&mut map, node26);
        Self::add_to_map(&mut map, node27);
        Self::add_to_map(&mut map, node28);
        //map[0].add_neighbour(map[1].clone(), Arc::new(NeighbourRelationship::new(0, Neighbourhood::IndustryPark)));
        //map[0].add_neighbour(map[2].clone(), Arc::new(NeighbourRelationship::new(1, Neighbourhood::IndustryPark)));
        //map[1].add_neighbour(map[2].clone(), Arc::new(NeighbourRelationship::new(2, Neighbourhood::IndustryPark)));
        //map[2].add_neighbour(map[3].clone(), Arc::new(NeighbourRelationship::new(3, Neighbourhood::Suburbs)));
        //map[3].add_neighbour(map[4].clone(), Arc::new(NeighbourRelationship::new(4, Neighbourhood::RingRoad)));
        //map[3].add_neighbour(map[9].clone(), Arc::new(NeighbourRelationship::new(5, Neighbourhood::RingRoad)));
        //map[4].add_neighbour(map[5].clone(), Arc::new(NeighbourRelationship::new(6, Neighbourhood::Port)));
        //map[4].add_neighbour(map[6].clone(), Arc::new(NeighbourRelationship::new(7, Neighbourhood::RingRoad)));
        //map[6].add_neighbour(map[13].clone(), Arc::new(NeighbourRelationship::new(8, Neighbourhood::RingRoad)));
        //map[6].add_neighbour(map[7].clone(), Arc::new(NeighbourRelationship::new(9, Neighbourhood::Suburbs)));
        //map[7].add_neighbour(map[8].clone(), Arc::new(NeighbourRelationship::new(10, Neighbourhood::Suburbs)));
        /*
        TODO: Add neighbour relations to nodes
              Remember to refer to issue 47 for anything involving path costs
        */
        Self {
            map,
        }
    }
}
