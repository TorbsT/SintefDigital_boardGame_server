use std::{
    collections::{hash_map, HashMap},
    sync::Mutex,
};

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
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
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
    #[serde(skip)]
    pub map: NodeMap,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: NodeID,
    pub name: String,
    // #[serde(skip)]
    // pub neighbours: Vec<(NodeID, NeighbourRelationship)>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NeighbourRelationship {
    pub to: NodeID,
    pub neighbourhood: Neighbourhood,
    pub movement_cost: MovementCost,
}

#[derive(Clone, Default, Debug)]
pub struct NodeMap {
    pub nodes: Vec<Node>,
    pub edges: HashMap<NodeID, Vec<NeighbourRelationship>>,
    pub neighbourhood_cost: HashMap<Neighbourhood, MovementCost>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewGameInfo {
    pub host: Player,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameStartInput {
    pub player_id: PlayerID,
    pub in_game_id: InGameID,
    pub game_id: GameID,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerObjectiveCard {
    pub start_node_id: NodeID,
    pub pick_up_node_id: NodeID,
    pub drop_off_node_id: NodeID,
    pub special_vehicle_type: Option<VehicleType>,
}

//// =============== Structs impls ===============
impl GameState {
    #[must_use]
    pub fn new(name: String, game_id: GameID) -> Self {
        Self {
            id: game_id,
            name,
            players: Vec::new(),
            is_lobby: true,
            actions: Vec::new(),
            current_players_turn: InGameID::Orchestrator,
            district_modifiers: Vec::new(),
            accessed_districts: Vec::new(),
            map: NodeMap::new_default(),
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

            let Some(neighbours) = self.map.get_neighbour_relationships_of_node_with_id(current_node_id) else {
                return Err(format!("There was no node with id {}!", current_node_id));
            };

            let Some(neighbour_relationship) = neighbours.iter().find(|relationship| relationship.to == to_node_id) else {
                return Err(format!("The node you are trying to go to is not a neighbour. From node with id {} to {}", current_node_id, to_node_id));
            };
            if !self
                .accessed_districts
                .contains(&neighbour_relationship.neighbourhood)
            {
                self.accessed_districts
                    .push(neighbour_relationship.neighbourhood);
                player.remaining_moves -= match self
                    .map
                    .first_time_in_district_cost(neighbour_relationship.clone())
                {
                    Ok(cost) => cost,
                    Err(e) => return Err(e),
                };

                for modifier in self.district_modifiers.iter() {
                    if modifier.district != neighbour_relationship.neighbourhood {
                        continue;
                    }
                    if let Some(movement_value) = modifier.associated_movement_value {
                        player.remaining_moves += movement_value;
                    }
                }
            } else {
                player.remaining_moves -= neighbour_relationship.movement_cost;
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

impl Neighbourhood {
    pub const fn first() -> Self {
        Self::IndustryPark
    }

    pub const fn next(&self) -> Option<Self> {
        match self {
            Self::IndustryPark => Some(Self::Port),
            Self::Port => Some(Self::Suburbs),
            Self::Suburbs => Some(Self::RingRoad),
            Self::RingRoad => Some(Self::CityCentre),
            Self::CityCentre => Some(Self::Airport),
            Self::Airport => None,
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
        Self { id, name }
    }
}

#[allow(clippy::unwrap_used)]
impl NeighbourRelationship {
    pub fn new(to: NodeID, neighbourhood: Neighbourhood, movement_cost: MovementCost) -> Self {
        Self {
            to,
            neighbourhood,
            movement_cost,
        }
    }
}

impl NodeMap {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: HashMap::new(),
            neighbourhood_cost: HashMap::new(),
        }
    }

    #[must_use]
    pub fn new_default() -> Self {
        let mut map = Self::new();

        let node0: Node = Node::new(0, String::from("Factory"));
        let node1: Node = Node::new(1, String::from("Refinery"));
        let node2: Node = Node::new(2, String::from("Industry Park"));
        let node3: Node = Node::new(3, String::from("I1"));
        let node4: Node = Node::new(4, String::from("I2"));
        let node5: Node = Node::new(5, String::from("Port"));
        let node6: Node = Node::new(6, String::from("I3"));
        let node7: Node = Node::new(7, String::from("Beach"));
        let node8: Node = Node::new(8, String::from("Northside"));
        let node9: Node = Node::new(9, String::from("I4"));
        let node10: Node = Node::new(10, String::from("Central Station"));
        let node11: Node = Node::new(11, String::from("City Square"));
        let node12: Node = Node::new(12, String::from("Concert Hall"));
        let node13: Node = Node::new(13, String::from("Eastside Mart"));
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

        map.nodes.push(node0.clone());
        map.nodes.push(node1.clone());
        map.nodes.push(node2.clone());
        map.nodes.push(node3.clone());
        map.nodes.push(node4.clone());
        map.nodes.push(node5.clone());
        map.nodes.push(node6.clone());
        map.nodes.push(node7.clone());
        map.nodes.push(node8.clone());
        map.nodes.push(node9.clone());
        map.nodes.push(node10.clone());
        map.nodes.push(node11.clone());
        map.nodes.push(node12.clone());
        map.nodes.push(node13.clone());
        map.nodes.push(node14.clone());
        map.nodes.push(node15.clone());
        map.nodes.push(node16.clone());
        map.nodes.push(node17.clone());
        map.nodes.push(node18.clone());
        map.nodes.push(node19.clone());
        map.nodes.push(node20.clone());
        map.nodes.push(node21.clone());
        map.nodes.push(node22.clone());
        map.nodes.push(node23.clone());
        map.nodes.push(node24.clone());
        map.nodes.push(node25.clone());
        map.nodes.push(node26.clone());
        map.nodes.push(node27.clone());
        map.nodes.push(node28.clone());

        map.add_relationship(node0, node1, Neighbourhood::IndustryPark, 0);
        map.add_relationship(node0, node2, Neighbourhood::IndustryPark, 0);
        map.add_relationship(node1, node2, Neighbourhood::IndustryPark, 0);
        map.add_relationship(node2, node3, Neighbourhood::Suburbs, 0);
        map.add_relationship(node3, node4, Neighbourhood::RingRoad, 0);
        map.add_relationship(node3, node9, Neighbourhood::RingRoad, 0);
        map.add_relationship(node4, node5, Neighbourhood::Port, 0);
        map.add_relationship(node4, node6, Neighbourhood::RingRoad, 0);
        map.add_relationship(node6, node13, Neighbourhood::RingRoad, 0);
        map.add_relationship(node6, node7, Neighbourhood::Suburbs, 0);
        map.add_relationship(node7, node8, Neighbourhood::Suburbs, 0);
        map.add_relationship(node9, node10, Neighbourhood::CityCentre, 0);
        map.add_relationship(node9, node18, Neighbourhood::RingRoad, 0);
        map.add_relationship(node10, node11, Neighbourhood::CityCentre, 0);
        map.add_relationship(node10, node15, Neighbourhood::CityCentre, 0);
        map.add_relationship(node11, node12, Neighbourhood::CityCentre, 0);
        map.add_relationship(node11, node16, Neighbourhood::CityCentre, 0);
        map.add_relationship(node12, node13, Neighbourhood::CityCentre, 0);
        map.add_relationship(node13, node14, Neighbourhood::Suburbs, 0);
        map.add_relationship(node13, node20, Neighbourhood::RingRoad, 0);
        map.add_relationship(node14, node21, Neighbourhood::Suburbs, 0);
        map.add_relationship(node15, node16, Neighbourhood::CityCentre, 0);
        map.add_relationship(node16, node19, Neighbourhood::CityCentre, 0);
        map.add_relationship(node17, node18, Neighbourhood::Suburbs, 0);
        map.add_relationship(node18, node19, Neighbourhood::RingRoad, 0);
        map.add_relationship(node18, node23, Neighbourhood::Suburbs, 0);
        map.add_relationship(node19, node20, Neighbourhood::RingRoad, 0);
        map.add_relationship(node20, node26, Neighbourhood::Suburbs, 0);
        map.add_relationship(node20, node27, Neighbourhood::Airport, 0);
        map.add_relationship(node21, node27, Neighbourhood::Airport, 0);
        map.add_relationship(node22, node23, Neighbourhood::Suburbs, 0);
        map.add_relationship(node23, node24, Neighbourhood::Suburbs, 0);
        map.add_relationship(node24, node25, Neighbourhood::Suburbs, 0);
        map.add_relationship(node25, node26, Neighbourhood::Suburbs, 0);
        map.add_relationship(node26, node27, Neighbourhood::Airport, 0);
        map.add_relationship(node27, node28, Neighbourhood::Airport, 0);

        let mut neighbourhood = Neighbourhood::first();
        map.change_neighbourhood_cost(neighbourhood, 1);
        while let Some(neighbourhood) = neighbourhood.next() {
            map.change_neighbourhood_cost(neighbourhood, 1);
        }

        map
    }

    pub fn get_node_by_id(&self, position_node_id: NodeID) -> Result<Node, String> {
        self.nodes
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

    pub fn get_neighbour_relationships_of_node_with_id(
        &self,
        node_id: NodeID,
    ) -> Option<Vec<NeighbourRelationship>> {
        match self.edges.get(&node_id) {
            Some(edge) => Some(edge.clone()),
            None => None,
        }
    }

    pub fn change_neighbourhood_cost(&mut self, neighbourhood: Neighbourhood, cost: MovementCost) {
        self.neighbourhood_cost.insert(neighbourhood, cost);
    }

    pub fn first_time_in_district_cost(
        &self,
        neighbour_relationship: NeighbourRelationship,
    ) -> Result<MovementCost, String> {
        let Some(neighbourhood_cost) = self.neighbourhood_cost.get(&neighbour_relationship.neighbourhood) else {
            return Err(format!("There was no neighbourhood_cost in the nodemap for neighbourhood {:?}", neighbour_relationship.neighbourhood));
        };
        Ok(neighbourhood_cost + neighbour_relationship.movement_cost)
    }

    fn add_relationship(
        &mut self,
        node1: Node,
        node2: Node,
        neighbourhood: Neighbourhood,
        cost: MovementCost,
    ) {
        let mut relationship = NeighbourRelationship::new(node2.id, neighbourhood, cost);
        self.edges
            .entry(node1.id)
            .or_insert(Vec::new())
            .push(relationship.clone());
        relationship.to = node1.id;
        self.edges
            .entry(node2.id)
            .or_insert(Vec::new())
            .push(relationship);
    }
}
