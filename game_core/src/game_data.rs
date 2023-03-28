use std::{sync::{Mutex}, collections::HashMap};

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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
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
    pub id: u8,
    pub name: String,
    #[serde(skip)]
    pub neighbours: Vec<(u8, NeighbourRelationship)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighbourRelationship {
    pub id: u8,
    pub neighbourhood: u8,
    pub group_cost: u8,
    pub individual_cost: u8,
}

#[derive(Clone)]
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
    pub player_id: i32,
    pub game_id: i32,
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

    pub fn move_player_with_id(&mut self, player_id: i32, to_node: Node) -> Result<(), String> {
        for player in self.players.iter_mut() {
            if player.unique_id != player_id {
                continue;
            }
            player.position = Some(to_node);
            // TODO: Add the ability to change role in the game aswell when applicable
            return Ok(());
        }
        Err("There were no players in this game that match the player to update".to_string())
    }

    pub fn update_game(&mut self, update: Self) {
        self.players = update.players;
    }

    pub fn get_player_with_unique_id(&self, player_id: i32) -> Result<Player, &str> {
        self.players
            .iter()
            .find(|p| p.unique_id == player_id)
            .map_or(
                Err("There is no player in the game with the given id"),
                |player| Ok(player.clone()),
            )
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
    pub const fn new(id: u8, name: String) -> Self {
        Self {
            id,
            name,
            neighbours: Vec::new(),
        }
    }

    pub fn add_neighbour(&mut self, neighbour: &mut Self, relationship: NeighbourRelationship) {
        self.neighbours.push((neighbour.id, relationship.clone()));
        neighbour.neighbours.push((self.id, relationship));
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum Neighbourhood {
    IndustryPark,
    Port,
    Suburbs,
    RingRoad,
    CityCentre,
    Airport,
}

lazy_static! {
    static ref GROUP_COST_MAP: Mutex<HashMap<Neighbourhood, u8>> =
        Mutex::new({
            let mut map = HashMap::new();
            map.insert(Neighbourhood::IndustryPark, 1);
            map.insert(Neighbourhood::Port, 1);
            map.insert(Neighbourhood::Suburbs, 1);
            map.insert(Neighbourhood::RingRoad, 1);
            map.insert(Neighbourhood::CityCentre, 1);
            map.insert(Neighbourhood::Airport, 1);
            map
        });
}

#[allow(clippy::unwrap_used)]
impl NeighbourRelationship {
    #[must_use]
    pub fn new(id: u8, neighbourhood_enum: Neighbourhood) -> Self {
        let group = GROUP_COST_MAP.lock().unwrap();
        let group_cost: u8 = *group.get(&neighbourhood_enum).unwrap();
        let neighbourhood: u8 = neighbourhood_enum as u8;
        Self {
            id,
            neighbourhood,
            group_cost,
            individual_cost: 0,
        }
    }

    pub fn update_individual_cost(&mut self, update: u8) {
        self.individual_cost = update;
    }

    pub const fn total_cost(&self) -> u8 {
        self.group_cost + self.individual_cost
    }
}

impl NodeMap {
    pub fn add_to_map(map: &mut Vec<Node>, node: Node) {
        map.push(node);
    }

    pub fn add_relationship(node1: &mut Node, node2: &mut Node, relationship: &mut NeighbourRelationship) {
        node1.add_neighbour(node2, relationship.clone());
    }

    #[allow(clippy::unwrap_used)]
    pub fn update_neighbour_costs(&mut self, neighbourhood_enum: Neighbourhood, value: u8) {
        let mut group_cost_map_reference = GROUP_COST_MAP.lock().unwrap();
        group_cost_map_reference.insert(neighbourhood_enum, value);
        for node in &mut self.map {
            for neighbour_relationship in &mut node.neighbours {
                if neighbour_relationship.1.neighbourhood == neighbourhood_enum as u8 {
                    neighbour_relationship.1.group_cost = value;
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn update_individual_cost(&mut self, node1_ID: u8, node2_ID: u8, value: u8) {
        self.update_individual_cost_recursion(node1_ID, node2_ID, value, false);
    }

    #[allow(non_snake_case)]
    fn update_individual_cost_recursion(&mut self, node1_ID: u8, node2_ID: u8, value: u8, updated_other_neighbour: bool) {
        let mut found_neighbour: bool = false;
        let node1 = &mut self.map[node1_ID as usize];
        for mut neighbour in &mut node1.neighbours {
            if neighbour.0 == node2_ID {
                found_neighbour = true;
                neighbour.1.individual_cost = value;
                break;
            }
        }
        if !found_neighbour {
            //TODO: Throw error
        }
        if !updated_other_neighbour {
            self.update_individual_cost_recursion(node2_ID, node1_ID, value, true);
        }
    }

    #[must_use]
    pub fn new() -> Self {
        let mut map: Vec<Node> = Vec::new();
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
        Self::add_relationship(&mut node0, &mut node1, &mut NeighbourRelationship::new(0, Neighbourhood::IndustryPark));
        Self::add_relationship(&mut node0, &mut node2, &mut NeighbourRelationship::new(1, Neighbourhood::IndustryPark));
        Self::add_relationship(&mut node1, &mut node2, &mut NeighbourRelationship::new(2, Neighbourhood::IndustryPark));
        Self::add_relationship(&mut node2, &mut node3, &mut NeighbourRelationship::new(3, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node3, &mut node4, &mut NeighbourRelationship::new(4, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node3, &mut node9, &mut NeighbourRelationship::new(5, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node4, &mut node5, &mut NeighbourRelationship::new(6, Neighbourhood::Port));
        Self::add_relationship(&mut node4, &mut node6, &mut NeighbourRelationship::new(7, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node6, &mut node13, &mut NeighbourRelationship::new(8, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node6, &mut node7, &mut NeighbourRelationship::new(9, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node7, &mut node8, &mut NeighbourRelationship::new(10, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node9, &mut node10, &mut NeighbourRelationship::new(11, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node9, &mut node18, &mut NeighbourRelationship::new(12, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node10, &mut node11, &mut NeighbourRelationship::new(13, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node10, &mut node15, &mut NeighbourRelationship::new(14, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node11, &mut node12, &mut NeighbourRelationship::new(15, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node11, &mut node16, &mut NeighbourRelationship::new(16, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node12, &mut node13, &mut NeighbourRelationship::new(17, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node13, &mut node14, &mut NeighbourRelationship::new(18, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node13, &mut node20, &mut NeighbourRelationship::new(19, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node14, &mut node21, &mut NeighbourRelationship::new(20, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node15, &mut node16, &mut NeighbourRelationship::new(21, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node16, &mut node19, &mut NeighbourRelationship::new(22, Neighbourhood::CityCentre));
        Self::add_relationship(&mut node17, &mut node18, &mut NeighbourRelationship::new(23, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node18, &mut node19, &mut NeighbourRelationship::new(24, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node18, &mut node23, &mut NeighbourRelationship::new(25, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node19, &mut node20, &mut NeighbourRelationship::new(26, Neighbourhood::RingRoad));
        Self::add_relationship(&mut node20, &mut node26, &mut NeighbourRelationship::new(27, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node20, &mut node27, &mut NeighbourRelationship::new(28, Neighbourhood::Airport));
        Self::add_relationship(&mut node21, &mut node27, &mut NeighbourRelationship::new(29, Neighbourhood::Airport));
        Self::add_relationship(&mut node22, &mut node23, &mut NeighbourRelationship::new(30, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node23, &mut node24, &mut NeighbourRelationship::new(31, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node24, &mut node25, &mut NeighbourRelationship::new(32, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node25, &mut node26, &mut NeighbourRelationship::new(33, Neighbourhood::Suburbs));
        Self::add_relationship(&mut node26, &mut node27, &mut NeighbourRelationship::new(34, Neighbourhood::Airport));
        Self::add_relationship(&mut node27, &mut node28, &mut NeighbourRelationship::new(35, Neighbourhood::Airport));
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
        Self {
            map,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::game_data::{Node, NeighbourRelationship};

    use super::*;

    #[test]
    fn test_node_add_neighbour() {
        let mut node0: Node = Node::new(0, String::from("First testing node"));
        let mut node1: Node = Node::new(1, String::from("Second testing node"));
        node0.add_neighbour(&mut node1, NeighbourRelationship::new(0, Neighbourhood::Suburbs));
        assert!(node0.neighbours[0].0 == 1);
        let group_cost_map_reference = GROUP_COST_MAP.lock().unwrap();
        assert!(node0.neighbours[0].1.group_cost == *group_cost_map_reference.get(&Neighbourhood::Suburbs).unwrap());
    }

    #[test]
    fn test_cost() {
        let mut node_map: NodeMap = NodeMap::new();
        assert!(node_map.map[0].neighbours[0].1.total_cost() == 1);
        node_map.update_neighbour_costs(Neighbourhood::IndustryPark, 2);
        assert!(node_map.map[0].neighbours[0].1.total_cost() == 2);
        assert!(node_map.map[27].neighbours[0].1.total_cost() == 1);
        node_map.update_individual_cost(0, 1, 2);
        assert!(node_map.map[0].neighbours[0].1.total_cost() == 4);
        assert!(node_map.map[1].neighbours[0].1.total_cost() == 4);
        assert!(node_map.map[1].neighbours[0].1.individual_cost == 2);
        assert!(node_map.map[1].neighbours[0].1.group_cost == 2);
        assert!(node_map.map[0].neighbours[1].1.total_cost() == 2);
        node_map.update_individual_cost(27, 28, 1);
        assert!(node_map.map[27].neighbours[3].1.total_cost() == 2);
        assert!(node_map.map[28].neighbours[0].1.total_cost() == 2);
    }

    #[test]
    fn test_nodemap_access() {
        let node_map: NodeMap = NodeMap::new();
        assert!(node_map.map.len() == 29);
        assert!(node_map.map[16].neighbours[0].0 == 11);
        for n in 0..node_map.map.len() {
            assert!(node_map.map[n].id == n as u8);
        }
    }
}
