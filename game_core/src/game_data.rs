use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};

//// =============== Types ===============
pub type NodeID = u8;
pub type PlayerID = i32;
pub type GameID = i32;
pub type NeighbourRelationshipID = u8;
pub type MovementCost = i16;
pub type MovementValue = MovementCost;
pub type MovesRemaining = MovementCost;
pub type Money = i32;

//// =============== Constants ===============
const MAX_PLAYER_COUNT: usize = 6; // TODO: UPDATE THIS IF INGAMEID IS UPDATED
pub const MAX_TOLL_MODIFIER_COUNT: usize = 1;
pub const MAX_ACCESS_MODIFIER_COUNT: usize = 2;
pub const MAX_PRIORITY_MODIFIER_COUNT: usize = 2;

//// =============== Enums ===============
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum InGameID {
    Undecided = 0,
    PlayerOne = 1,
    PlayerTwo = 2,
    PlayerThree = 3,
    PlayerFour = 4,
    PlayerFive = 5,
    Orchestrator = 6,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Debug)]
pub enum PlayerInputType {
    Movement,
    ChangeRole,
    All,
    NextTurn,
    UndoAction,
    ModifyDistrict,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Neighbourhood {
    IndustryPark,
    Port,
    Suburbs,
    RingRoad,
    CityCentre,
    Airport,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum VehicleType {
    Electric,
    Buss,
    Emergency,
    Industrial,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DistrictModifierType {
    Access,
    Priority,
    Toll,
}

//// =============== Structs ===============
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub id: GameID,
    pub name: String,
    pub players: Vec<Player>,
    pub is_lobby: bool,
    pub current_players_turn: InGameID,
    pub district_modifiers: Vec<DistrictModifier>,
    #[serde(skip)]
    pub actions: Vec<PlayerInput>,
    #[serde(skip)]
    pub accessed_districts: Vec<Neighbourhood>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub connected_game_id: Option<GameID>,
    pub in_game_id: InGameID,
    pub unique_id: PlayerID,
    pub name: String,
    pub position_node_id: Option<NodeID>,
    pub remaining_moves: MovesRemaining,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeID,
    pub name: String,
    #[serde(skip)]
    pub neighbours: Vec<(NodeID, NeighbourRelationship)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighbourRelationship {
    pub id: NeighbourRelationshipID,
    pub neighbourhood: Neighbourhood,
    pub group_cost: MovementCost,
    pub individual_cost: MovementCost,
}

#[derive(Clone, Default)]
pub struct NodeMap {
    pub map: Vec<Node>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewGameInfo {
    pub host: Player,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub player_id: PlayerID,
    pub game_id: GameID,
    pub input_type: PlayerInputType,
    pub related_role: Option<InGameID>,
    pub related_node_id: Option<NodeID>,
    pub district_modifier: Option<DistrictModifier>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DistrictModifier {
    pub district: Neighbourhood,
    pub modifier: DistrictModifierType,
    pub vehicle_type: Option<VehicleType>,
    pub associated_movement_value: Option<MovementValue>,
    pub associated_money_value: Option<Money>,
    pub delete: bool,
}

//// =============== Structs impls ===============
impl GameState {
    #[must_use]
    pub const fn new(name: String, game_id: GameID) -> Self {
        Self {
            id: game_id,
            name,
            players: Vec::new(),
            is_lobby: true,
            actions: Vec::new(),
            current_players_turn: InGameID::Orchestrator,
            district_modifiers: Vec::new(),
            accessed_districts: Vec::new(),
        }
    }

    pub fn contains_player_with_unique_id(&self, unique_id: PlayerID) -> bool {
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

    pub fn move_player_with_id(
        &mut self,
        player_id: PlayerID,
        to_node_id: NodeID,
    ) -> Result<(), String> {
        for player in self.players.iter_mut() {
            if player.unique_id != player_id {
                continue;
            }

            let Some(current_node_id) = player.position_node_id else {
                return Err("The player is not at any node!".to_string());
            };

            let current_node = match NodeMap::new().get_node_by_id(current_node_id) {
                Ok(node) => node,
                Err(e) => return Err(e),
            };

            let Some((_, neighbour_relationship)) = current_node.neighbours.iter().find(|(node_id, _)| node_id == &to_node_id) else {
                return Err(format!("The node you are trying to go to is not a neighbour. From node with id {} to {}", current_node_id, to_node_id));
            };
            if !self
                .accessed_districts
                .contains(&neighbour_relationship.neighbourhood)
            {
                self.accessed_districts
                    .push(neighbour_relationship.neighbourhood);
                player.remaining_moves -= neighbour_relationship.total_cost();
                for modifier in self.district_modifiers.iter() {
                    if modifier.district != neighbour_relationship.neighbourhood {
                        continue;
                    }
                    if let Some(movement_value) = modifier.associated_movement_value {
                        player.remaining_moves += movement_value;
                    }
                }
            } else {
                player.remaining_moves -= neighbour_relationship.individual_cost;
            }
            player.position_node_id = Some(to_node_id);
            return Ok(());
        }
        Err("There were no players in this game that match the player to update".to_string())
    }

    pub fn update_game(&mut self, update: Self) {
        self.players = update.players;
    }

    pub fn assign_player_role(&mut self, change_info: (PlayerID, InGameID)) -> Result<(), &str> {
        let (related_player_id, change_to_role) = change_info;
        if self
            .players
            .iter()
            .any(|p| p.in_game_id == change_to_role && change_to_role != InGameID::Undecided)
        {
            return Err("There is already a player with this role");
        }

        for player in self.players.iter_mut() {
            if player.unique_id != related_player_id {
                continue;
            }
            player.in_game_id = change_to_role;
            return Ok(());
        }
        Err("There were no players in this game that match the player to update")
    }

    pub fn get_player_with_unique_id(&self, player_id: PlayerID) -> Result<Player, &str> {
        self.players
            .iter()
            .find(|p| p.unique_id == player_id)
            .map_or(
                Err("There is no player in the game with the given id"),
                |player| Ok(player.clone()),
            )
    }

    pub fn remove_player_with_id(&mut self, player_id: i32) {
        self.players.retain(|player| player.unique_id != player_id);
    }

    pub fn next_player_turn(&mut self) {
        let mut next_player_turn = self.current_players_turn.next();
        let mut counter = 0;
        while !self
            .players
            .iter()
            .any(|p| p.in_game_id == next_player_turn)
        {
            next_player_turn = next_player_turn.next();
            if counter >= 1000 {
                next_player_turn = InGameID::Orchestrator;
                break;
            }
            counter += 1;
        }

        self.current_players_turn = next_player_turn;
    }
}

impl InGameID {
    pub const fn next(&self) -> Self {
        match self {
            Self::Undecided => Self::Orchestrator,
            Self::PlayerOne => Self::PlayerTwo,
            Self::PlayerTwo => Self::PlayerThree,
            Self::PlayerThree => Self::PlayerFour,
            Self::PlayerFour => Self::PlayerFive,
            Self::PlayerFive => Self::Orchestrator,
            Self::Orchestrator => Self::PlayerOne,
        }
    }
}

impl Player {
    #[must_use]
    pub const fn new(unique_id: PlayerID, name: String) -> Self {
        Self {
            connected_game_id: None,
            in_game_id: InGameID::Undecided,
            unique_id,
            name,
            position_node_id: None,
            remaining_moves: 0,
        }
    }
}

impl Node {
    #[must_use]
    pub const fn new(id: NodeID, name: String) -> Self {
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

    pub fn has_neighbour_with_id(&self, neighbour_id: NodeID) -> bool {
        self.neighbours.iter().any(|(id, _)| *id == neighbour_id)
    }

    pub fn get_movement_cost_to_neighbour_with_id(
        &self,
        neighbour_id: NodeID,
    ) -> Result<MovementCost, String> {
        if !self.has_neighbour_with_id(neighbour_id) {
            return Err(format!(
                "Node {} does not have a neighbour with id {}",
                self.id, neighbour_id
            ));
        }

        self.neighbours
            .iter()
            .find(|(id, _)| *id == neighbour_id)
            .map_or_else(
                || {
                    Err(format!(
                        "Node {} could not find a cost to the neighbour with id {}",
                        self.id, neighbour_id
                    ))
                },
                |(_, relationship)| Ok(relationship.total_cost()),
            )
    }
}

lazy_static! {
    static ref GROUP_COST_MAP: Mutex<HashMap<Neighbourhood, MovementCost>> = Mutex::new({
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
    pub fn new(id: NeighbourRelationshipID, neighbourhood_enum: Neighbourhood) -> Self {
        let group = GROUP_COST_MAP.lock().unwrap();
        let group_cost: MovementCost = *group.get(&neighbourhood_enum).unwrap();
        let neighbourhood = neighbourhood_enum;
        Self {
            id,
            neighbourhood,
            group_cost,
            individual_cost: 0,
        }
    }

    pub fn update_individual_cost(&mut self, update: MovementCost) {
        self.individual_cost = update;
    }

    pub const fn total_cost(&self) -> MovementCost {
        self.group_cost + self.individual_cost
    }
}

impl NodeMap {
    pub fn add_to_map(map: &mut Vec<Node>, node: Node) {
        map.push(node);
    }

    pub fn add_relationship(
        node1: &mut Node,
        node2: &mut Node,
        relationship: &mut NeighbourRelationship,
    ) {
        node1.add_neighbour(node2, relationship.clone());
    }

    #[allow(clippy::unwrap_used)]
    pub fn update_neighbour_costs(
        &mut self,
        neighbourhood_enum: Neighbourhood,
        value: MovementCost,
    ) {
        let mut group_cost_map_reference = GROUP_COST_MAP.lock().unwrap();
        group_cost_map_reference.insert(neighbourhood_enum, value);
        for node in &mut self.map {
            for neighbour_relationship in &mut node.neighbours {
                if neighbour_relationship.1.neighbourhood == neighbourhood_enum {
                    neighbour_relationship.1.group_cost = value;
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn update_individual_cost(
        &mut self,
        node1_ID: NodeID,
        node2_ID: NodeID,
        value: MovementCost,
    ) {
        self.update_individual_cost_recursion(node1_ID, node2_ID, value, false);
    }

    #[allow(non_snake_case)]
    fn update_individual_cost_recursion(
        &mut self,
        node1_ID: NodeID,
        node2_ID: NodeID,
        value: MovementCost,
        updated_other_neighbour: bool,
    ) {
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
        Self::add_relationship(
            &mut node0,
            &mut node1,
            &mut NeighbourRelationship::new(0, Neighbourhood::IndustryPark),
        );
        Self::add_relationship(
            &mut node0,
            &mut node2,
            &mut NeighbourRelationship::new(1, Neighbourhood::IndustryPark),
        );
        Self::add_relationship(
            &mut node1,
            &mut node2,
            &mut NeighbourRelationship::new(2, Neighbourhood::IndustryPark),
        );
        Self::add_relationship(
            &mut node2,
            &mut node3,
            &mut NeighbourRelationship::new(3, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node3,
            &mut node4,
            &mut NeighbourRelationship::new(4, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node3,
            &mut node9,
            &mut NeighbourRelationship::new(5, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node4,
            &mut node5,
            &mut NeighbourRelationship::new(6, Neighbourhood::Port),
        );
        Self::add_relationship(
            &mut node4,
            &mut node6,
            &mut NeighbourRelationship::new(7, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node6,
            &mut node13,
            &mut NeighbourRelationship::new(8, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node6,
            &mut node7,
            &mut NeighbourRelationship::new(9, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node7,
            &mut node8,
            &mut NeighbourRelationship::new(10, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node9,
            &mut node10,
            &mut NeighbourRelationship::new(11, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node9,
            &mut node18,
            &mut NeighbourRelationship::new(12, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node10,
            &mut node11,
            &mut NeighbourRelationship::new(13, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node10,
            &mut node15,
            &mut NeighbourRelationship::new(14, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node11,
            &mut node12,
            &mut NeighbourRelationship::new(15, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node11,
            &mut node16,
            &mut NeighbourRelationship::new(16, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node12,
            &mut node13,
            &mut NeighbourRelationship::new(17, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node13,
            &mut node14,
            &mut NeighbourRelationship::new(18, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node13,
            &mut node20,
            &mut NeighbourRelationship::new(19, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node14,
            &mut node21,
            &mut NeighbourRelationship::new(20, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node15,
            &mut node16,
            &mut NeighbourRelationship::new(21, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node16,
            &mut node19,
            &mut NeighbourRelationship::new(22, Neighbourhood::CityCentre),
        );
        Self::add_relationship(
            &mut node17,
            &mut node18,
            &mut NeighbourRelationship::new(23, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node18,
            &mut node19,
            &mut NeighbourRelationship::new(24, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node18,
            &mut node23,
            &mut NeighbourRelationship::new(25, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node19,
            &mut node20,
            &mut NeighbourRelationship::new(26, Neighbourhood::RingRoad),
        );
        Self::add_relationship(
            &mut node20,
            &mut node26,
            &mut NeighbourRelationship::new(27, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node20,
            &mut node27,
            &mut NeighbourRelationship::new(28, Neighbourhood::Airport),
        );
        Self::add_relationship(
            &mut node21,
            &mut node27,
            &mut NeighbourRelationship::new(29, Neighbourhood::Airport),
        );
        Self::add_relationship(
            &mut node22,
            &mut node23,
            &mut NeighbourRelationship::new(30, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node23,
            &mut node24,
            &mut NeighbourRelationship::new(31, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node24,
            &mut node25,
            &mut NeighbourRelationship::new(32, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node25,
            &mut node26,
            &mut NeighbourRelationship::new(33, Neighbourhood::Suburbs),
        );
        Self::add_relationship(
            &mut node26,
            &mut node27,
            &mut NeighbourRelationship::new(34, Neighbourhood::Airport),
        );
        Self::add_relationship(
            &mut node27,
            &mut node28,
            &mut NeighbourRelationship::new(35, Neighbourhood::Airport),
        );
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
        Self { map }
    }

    pub fn get_node_by_id(&self, position_node_id: NodeID) -> Result<Node, String> {
        self.map
            .iter()
            .find(|&node| node.id == position_node_id)
            .map_or_else(
                || {
                    Err(format!(
                        "There is no node with the given ID: {}",
                        position_node_id
                    ))
                },
                |node| Ok(node.clone()),
            )
    }
}

#[cfg(test)]
mod tests {

    use crate::game_data::{NeighbourRelationship, Node};

    use super::*;

    #[test]
    fn test_node_add_neighbour() {
        let mut node0: Node = Node::new(0, String::from("First testing node"));
        let mut node1: Node = Node::new(1, String::from("Second testing node"));
        node0.add_neighbour(
            &mut node1,
            NeighbourRelationship::new(0, Neighbourhood::Suburbs),
        );
        assert!(node0.neighbours[0].0 == 1);
        let group_cost_map_reference = GROUP_COST_MAP.lock().unwrap();
        assert!(
            node0.neighbours[0].1.group_cost
                == *group_cost_map_reference
                    .get(&Neighbourhood::Suburbs)
                    .unwrap()
        );
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

impl PlayerInput {
    #[must_use]
    pub const fn new(player_id: PlayerID, game_id: GameID, input_type: PlayerInputType) -> Self {
        Self {
            input_type,
            related_role: None,
            related_node_id: None,
            player_id,
            game_id,
            district_modifier: None,
        }
    }
}
