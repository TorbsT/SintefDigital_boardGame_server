use std::{cmp, collections::HashMap, mem};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::situation_card_list::situation_card_list;

//// =============== Types ===============
pub type NodeID = u8;
pub type PlayerID = i32;
pub type GameID = i32;
pub type NeighbourRelationshipID = u8;
pub type MovementCost = i16;
pub type MovementValue = MovementCost;
pub type MovesRemaining = MovementCost;
pub type Money = i32;
pub type SituationCardID = u8;

//// =============== Constants ===============
const MAX_PLAYER_COUNT: usize = 6; // TODO: UPDATE THIS IF INGAMEID IS UPDATED
pub const MAX_TOLL_MODIFIER_COUNT: usize = 1;
pub const MAX_ACCESS_MODIFIER_COUNT: usize = 2;
pub const MAX_PRIORITY_MODIFIER_COUNT: usize = 2;
pub const START_MOVEMENT_AMOUNT: MovementValue = 8;

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

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Traffic {
    LevelOne,
    LevelTwo,
    LevelThree,
    LevelFour,
    LevelFive,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Debug)]
pub enum PlayerInputType {
    Movement,
    ChangeRole,
    All,
    NextTurn,
    UndoAction,
    ModifyDistrict,
    StartGame,
    AssignSituationCard,
    LeaveGame,
    DropPackageAtTrainStation,
    ModifyParkAndRide,
    UseParkAndRide,
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
    Normal,
    Geolocation,
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
    pub situation_card: Option<SituationCard>,
    pub park_and_ride_nodes: Vec<ParkAndRideEdge>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ParkAndRideEdge {
    pub node_one: NodeID,
    pub node_two: NodeID,
    pub delete: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub connected_game_id: Option<GameID>,
    pub in_game_id: InGameID,
    pub unique_id: PlayerID,
    pub name: String,
    pub position_node_id: Option<NodeID>,
    pub remaining_moves: MovesRemaining,
    pub objective_card: Option<PlayerObjectiveCard>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: NodeID,
    pub name: String,
    pub is_connected_to_rail: bool,
    pub is_parking_spot: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NeighbourRelationship {
    pub to: NodeID,
    pub neighbourhood: Neighbourhood,
    pub movement_cost: MovementCost,
    pub is_park_and_ride: bool,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub player_id: PlayerID,
    pub game_id: GameID,
    pub input_type: PlayerInputType,
    pub related_role: Option<InGameID>,
    pub related_node_id: Option<NodeID>,
    pub district_modifier: Option<DistrictModifier>,
    pub situation_card_id: Option<SituationCardID>,
    pub park_and_ride_modifier: Option<ParkAndRideEdge>,
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PlayerObjectiveCard {
    pub start_node_id: NodeID,
    pub pick_up_node_id: NodeID,
    pub drop_off_node_id: NodeID,
    pub special_vehicle_types: Vec<VehicleType>,
    pub picked_package_up: bool,
    pub dropped_package_off: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CostTuple {
    neighbourhood: Neighbourhood,
    traffic: Traffic,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SituationCard {
    pub card_id: SituationCardID,
    pub title: String,
    pub description: String,
    pub goal: String,
    pub costs: Vec<CostTuple>,
    pub objective_cards: Vec<PlayerObjectiveCard>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SituationCardList {
    pub situation_cards: Vec<SituationCard>,
}

//// =============== Structs impls ===============
impl GameState {
    //TODO: Alle max checks, som f.eks. maks antall spillere i en lobby og maks antall restrictions burde flyttes til game_rule_checker
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
            situation_card: None,
            park_and_ride_nodes: Vec::new(),
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

                let mut bonus_moves = 0;

                if let Some(obj_card) = player.objective_card.clone() {
                    for modifier in self.district_modifiers.iter() {
                        if modifier.district != neighbour_relationship.neighbourhood {
                            continue;
                        }

                        let Some(vehicle_type) = modifier.vehicle_type else {
                            continue;
                        };

                        if !obj_card.special_vehicle_types.contains(&vehicle_type) {
                            continue;
                        }

                        if let Some(movement_value) = modifier.associated_movement_value {
                            bonus_moves = cmp::max(bonus_moves, movement_value);
                        }
                    }
                }
                player.remaining_moves += bonus_moves;
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
        if self
            .players
            .iter()
            .all(|player| player.in_game_id != InGameID::Orchestrator)
        {
            if let Some(mut p) = self.players.first_mut() {
                p.in_game_id = InGameID::Orchestrator;
                p.objective_card = None;
            };
        }
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
            if next_player_turn == InGameID::Orchestrator {
                self.reset_player_movement_values();
            }
            if counter >= 1000 {
                next_player_turn = InGameID::Orchestrator;
                break;
            }
            counter += 1;
        }

        self.current_players_turn = next_player_turn;
    }

    pub const fn get_starting_player_movement_value() -> MovementValue {
        START_MOVEMENT_AMOUNT
    }

    pub fn assign_random_situation_card_to_players(&mut self) -> Result<(), String> {
        let Some(situation_card) = self.situation_card.clone() else {
            return Err("The game does not have a situation card and can therefore not assign objective cards to the players!".to_string());
        };
        let mut objective_cards = situation_card.objective_cards;
        let mut rng = rand::thread_rng();
        for mut player in self.players.iter_mut() {
            if player.in_game_id == InGameID::Orchestrator {
                continue;
            }
            if objective_cards.is_empty() {
                return Err(
                    "There were not enough objective cards for all the players!".to_string()
                );
            }
            let index = rng.gen_range(0..objective_cards.len());
            let objective_card = objective_cards.remove(index);
            player.position_node_id = Some(objective_card.start_node_id);
            player.objective_card = Some(objective_card);
        }
        Ok(())
    }

    pub fn update_situation_card(&mut self, new_situation_card: SituationCard) {
        self.situation_card = Some(new_situation_card);
    }

    pub fn update_objective_status(&mut self) -> Result<(), String> {
        for player in self.players.iter_mut() {
            if player.in_game_id == InGameID::Orchestrator {
                continue;
            }
            let Some(player_position_id) = player.position_node_id else {
                return Err("The player did not have a position on the gameboard!".to_string());
            };
            let Some(mut objective_card) = player.objective_card.clone() else {
                return Err("The player did not have an objective card!".to_string());
            };
            if player_position_id == objective_card.pick_up_node_id {
                objective_card.picked_package_up = true;
            }
            if player_position_id == objective_card.drop_off_node_id
            && objective_card.picked_package_up
            {
                objective_card.dropped_package_off = true;
            }
            mem::swap(&mut player.objective_card, &mut Some(objective_card));
        }
        Ok(())
    }

    pub fn drop_package_at_train_station(&mut self) -> Result<(), String> {
        for player in self.players.iter_mut() {
            let Some(player_position_id) = player.position_node_id else {
                return Err("The player did not have a position on the gameboard!".to_string());
            };
            let Some(mut objective_card) = player.objective_card.clone() else {
                return Err("The player did not have an objective card!".to_string());
            };
            let player_node =  match self.map.get_node_by_id(player_position_id as NodeID) {
                    Ok(n) => n,
                Err(_) => return Err("Player node can not be determined while attempting to drop package!".to_string()),
            };
            let package_node = match self.map.get_node_by_id(objective_card.drop_off_node_id as NodeID) {
                Ok(n) => n,
                Err(_) => return Err("Destination node can not be determined while attempting to drop package!".to_string()),
            };
            if player_node.is_connected_to_rail
            && objective_card.picked_package_up
            && package_node.is_connected_to_rail
            {
                objective_card.dropped_package_off = true;
            }
            mem::swap(&mut player.objective_card, &mut Some(objective_card));
        }
        Ok(())
    }

    pub fn start_game(&mut self) -> Result<(), String> {
        let mut can_start_game = false;
        let mut errormessage =
            String::from("Unable to start game because lobby does not have an orchestrator");
        for player in self.players.clone() {
            if player.in_game_id == InGameID::Orchestrator {
                if self.players.len() < 2 {
                    errormessage =
                        "Unable to start game because there are not enough players".to_string();
                    break;
                };
                if self.situation_card.is_none() {
                    errormessage =
                        "Unable to start game because a situation card is not chosen".to_string();
                    break;
                }
                match self.assign_random_situation_card_to_players() {
                    Ok(_) => (),
                    Err(e) => {
                        errormessage = e;
                        break;
                    }
                }
                match self.update_objective_status() {
                    Ok(_) => (),
                    Err(e) => {
                        errormessage = e;
                        break;
                    }
                }
                can_start_game = true;
                self.is_lobby = false;
                break;
            }
        }
        match can_start_game {
            true => {
                self.reset_player_movement_values();
                Ok(())
            }
            false => Err(errormessage),
        }
    }

    pub fn update_node_map_with_situation_card(&mut self) -> Result<(), String> {
        match &self.situation_card {
            Some(card) => {
                self.map.update_neighbourhood_cost(card.clone());
                Ok(())
            },
            None => Err("Error: No situation card was assigned to the game, and therefore can not update nodemap costs".to_string()),
        }
    }

    pub fn reset_player_movement_values(&mut self) {
        self.players
            .iter_mut()
            .for_each(|player| player.remaining_moves = Self::get_starting_player_movement_value());
    }

    pub fn add_district_modifier(
        &mut self,
        district_modifier: DistrictModifier,
    ) -> Result<(), String> {
        let max_amount: usize = match district_modifier.modifier {
            DistrictModifierType::Access => MAX_ACCESS_MODIFIER_COUNT,
            DistrictModifierType::Priority => MAX_PRIORITY_MODIFIER_COUNT,
            DistrictModifierType::Toll => MAX_TOLL_MODIFIER_COUNT,
        };

        if max_amount
            <= self
                .district_modifiers
                .iter()
                .filter(|m| {
                    m.modifier == district_modifier.modifier
                        && m.district == district_modifier.district
                })
                .count()
        {
            return Err(format!("Cannot add more modifiers of type {:?} because there are already {} modifiers of that type!", district_modifier.modifier, max_amount));
        }

        self.district_modifiers.push(district_modifier);
        self.update_traffic_levels()
    }

    pub fn remove_district_modifier(
        &mut self,
        district_modifier: DistrictModifier,
    ) -> Result<(), String> {
        let mut distr_mod = district_modifier;
        distr_mod.delete = false;
        let Some(mod_pos) = self.district_modifiers.iter().position(|d_m| d_m == &distr_mod) else {
            return Err("There is no modifier like the given one in the game!".to_string());
        };
        self.district_modifiers.remove(mod_pos);
        self.update_traffic_levels()
    }

    fn update_traffic_levels(&mut self) -> Result<(), String> {
        let Some(mut situation_card) = self.situation_card.clone() else {
            return Err("There is no situation card in this game and it's therefore not possible to update the traffic levels!".to_string());
        };

        let mut new_cost_tuples = Vec::new();

        let situation_cards = situation_card_list();
        let Some(original_card) = situation_cards.iter().find(|c| c.card_id == situation_card.card_id) else {
            return Err("The situation card in the game has an ID was not found in the list of situation cards!".to_string());
        };
        let original_costs = original_card.costs.clone();

        for cost_tuple in original_costs {
            let mut new_cost_tuple = cost_tuple.clone();
            let mut is_access_modifier_used = false;
            let mut times_to_increase_when_access = -1;
            for modifier in self.district_modifiers.clone() {
                if modifier.district != cost_tuple.neighbourhood
                    || modifier.modifier != DistrictModifierType::Access
                {
                    continue;
                }

                let Some(vehicle_type) = modifier.vehicle_type else {
                    return Err("There was no vehicle type associated with the access modifier and can therefore not update the traffic levels!".to_string());
                };

                if !is_access_modifier_used {
                    new_cost_tuple.traffic = Traffic::LevelOne;
                    is_access_modifier_used = true;
                }

                times_to_increase_when_access += vehicle_type.times_to_increase_traffic_when_access() as i32;
            }

            for _ in 0..cmp::max(0, times_to_increase_when_access) {
                new_cost_tuple.traffic = new_cost_tuple.traffic.increased();
            }
            
            new_cost_tuples.push(new_cost_tuple);
        }

        situation_card.costs = new_cost_tuples;
        self.situation_card = Some(situation_card);

        Ok(())
    }

    pub fn add_park_and_ride(
        &mut self,
        node_id_one: NodeID,
        node_id_two: NodeID,
    ) -> Result<(), String> {
        match self.map.add_park_and_ride_to_edge(node_id_one, node_id_two) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        self.park_and_ride_nodes
            .push(ParkAndRideEdge::new(node_id_one, node_id_two));
        Ok(())
    }

    pub fn remove_park_and_ride(
        &mut self,
        node_id_one: NodeID,
        node_id_two: NodeID,
    ) -> Result<(), String> {
        match self
            .map
            .remove_park_and_ride_from_edge(node_id_one, node_id_two)
        {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        self.park_and_ride_nodes.retain(|nodes| {
            !((nodes.node_one == node_id_one && nodes.node_two == node_id_two)
                || (nodes.node_one == node_id_two && nodes.node_two == node_id_one))
        });
        Ok(())
    }

    pub fn use_park_and_ride_to_deliver_package(&mut self, player_id: PlayerID, destination_node_id: NodeID) -> Result<(), String> {
        let Some(player) = self.players.iter_mut().find(|p| p.unique_id == player_id) else {
            return Err(format!("There is no player with the id {} in this game!", player_id));
        };

        let Some(mut objective_card) = player.objective_card.clone() else {
            return Err("There is no objective card in this game and therefore the player can not use the park and ride!".to_string());
        };

        if !objective_card.picked_package_up {
            return Err("The player has not picked up the package yet and can therefore not use the park and ride!".to_string());
        }
        let Some(current_player_position_id) = player.position_node_id else {
            return Err("The player has no current position and can therefore not use the park and ride!".to_string());
        };
        let movement_cost = match self.map.cost_to_use_park_and_ride_to_deliver(current_player_position_id, destination_node_id) {
            Ok(cost) => cost,
            Err(e) => return Err(e),
        };
        player.remaining_moves -= movement_cost;

        objective_card.dropped_package_off = true;

        mem::swap(&mut player.objective_card, &mut Some(objective_card));

        Ok(())
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
            objective_card: None,
        }
    }
}

impl Node {
    #[must_use]
    pub const fn new(id: NodeID, name: String) -> Self {
        Self {
            id,
            name,
            is_parking_spot: false,
            is_connected_to_rail: false,
        }
    }

    pub fn toggle_rail_connection(&mut self) {
        self.is_connected_to_rail = !self.is_connected_to_rail;
    }
}

impl ParkAndRideEdge {
    pub const fn new(node_id_one: NodeID, node_id_two: NodeID) -> Self {
        Self {
            node_one: node_id_one,
            node_two: node_id_two,
            delete: false,
        }
    }
}

#[allow(clippy::unwrap_used)]
impl NeighbourRelationship {
    pub const fn new(
        to: NodeID,
        neighbourhood: Neighbourhood,
        movement_cost: MovementCost,
    ) -> Self {
        Self {
            to,
            neighbourhood,
            movement_cost,
            is_park_and_ride: false,
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

    pub fn update_neighbourhood_cost(&mut self, situation_card: SituationCard) {
        for i in situation_card.costs {
            self.neighbourhood_cost
                .insert(i.neighbourhood, i.traffic.get_movement_cost());
        }
    }

    #[must_use]
    pub fn new_default() -> Self {
        let mut map = Self::new();

        let node0: Node = Node::new(0, String::from("Factory"));
        let node1: Node = Node::new(1, String::from("Refinery"));
        let mut node2: Node = Node::new(2, String::from("Industry Park"));
        
        let node3: Node = Node::new(3, String::from("I1"));
        let node4: Node = Node::new(4, String::from("I2"));
        let node5: Node = Node::new(5, String::from("Port"));
        let node6: Node = Node::new(6, String::from("I3"));
        let node7: Node = Node::new(7, String::from("Beach"));
        let node8: Node = Node::new(8, String::from("Northside"));
        let mut node9: Node = Node::new(9, String::from("I4"));
        let mut node10: Node = Node::new(10, String::from("Central Station"));
        let node11: Node = Node::new(11, String::from("City Square"));
        let node12: Node = Node::new(12, String::from("Concert Hall"));
        let mut node13: Node = Node::new(13, String::from("Eastside Mart"));
        let node14: Node = Node::new(14, String::from("East Town"));
        let node15: Node = Node::new(15, String::from("Food Court"));
        let node16: Node = Node::new(16, String::from("City Park"));
        let node17: Node = Node::new(17, String::from("Quarry"));
        let node18: Node = Node::new(18, String::from("I5"));
        let mut node19: Node = Node::new(19, String::from("I6"));
        let node20: Node = Node::new(20, String::from("I7"));
        let mut node21: Node = Node::new(21, String::from("I8"));
        let node22: Node = Node::new(22, String::from("West Town"));
        let node23: Node = Node::new(23, String::from("Lakeside"));
        let mut node24: Node = Node::new(24, String::from("Warehouses"));
        let node25: Node = Node::new(25, String::from("I9"));
        let mut node26: Node = Node::new(26, String::from("I10"));
        let mut node27: Node = Node::new(27, String::from("Terminal 1"));
        let node28: Node = Node::new(28, String::from("Terminal 2"));

        node2.toggle_rail_connection();
        node10.toggle_rail_connection();
        node24.toggle_rail_connection();
        node27.toggle_rail_connection();

        node2.is_parking_spot = true;
        node9.is_parking_spot = true;
        node13.is_parking_spot = true;
        node19.is_parking_spot = true;
        node21.is_parking_spot = true;
        node26.is_parking_spot = true;
        node27.is_parking_spot = true;

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

        map.add_relationship(node0.clone(), node1.clone(), Neighbourhood::IndustryPark, 1);
        map.add_relationship(node0, node2.clone(), Neighbourhood::IndustryPark, 1);
        map.add_relationship(node1, node2.clone(), Neighbourhood::IndustryPark, 1);
        map.add_relationship(node2, node3.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node3.clone(), node4.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node3, node9.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node4.clone(), node5, Neighbourhood::Port, 1);
        map.add_relationship(node4, node6.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node6.clone(), node13.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node6, node7.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node7, node8, Neighbourhood::Suburbs, 1);
        map.add_relationship(node9.clone(), node10.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node9, node18.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node10.clone(), node11.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node10, node15.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node11.clone(), node12.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node11, node16.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node12, node13.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node13.clone(), node14.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node13, node20.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node14, node21.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node15, node16.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node16, node19.clone(), Neighbourhood::CityCentre, 1);
        map.add_relationship(node17, node18.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node18.clone(), node19.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node18, node23.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node19, node20.clone(), Neighbourhood::RingRoad, 1);
        map.add_relationship(node20.clone(), node26.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node20, node27.clone(), Neighbourhood::Airport, 1);
        map.add_relationship(node21, node27.clone(), Neighbourhood::Airport, 1);
        map.add_relationship(node22, node23.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node23, node24.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node24, node25.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node25, node26.clone(), Neighbourhood::Suburbs, 1);
        map.add_relationship(node26, node27.clone(), Neighbourhood::Airport, 1);
        map.add_relationship(node27, node28, Neighbourhood::Airport, 1);

        let mut neighbourhood = Neighbourhood::first();
        map.change_neighbourhood_cost(neighbourhood, 1);
        while let Some(n) = neighbourhood.next() {
            neighbourhood = n;
            map.change_neighbourhood_cost(n, 1);
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
        self.edges.get(&node_id).cloned()
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
        println!(
            "First time in district: {:?}",
            neighbour_relationship.neighbourhood
        );
        Ok(cmp::max(
            *neighbourhood_cost,
            neighbour_relationship.movement_cost,
        ))
    }

    pub fn are_nodes_neighbours(&self, node_1: NodeID, node_2: NodeID) -> Result<bool, String> {
        let Some(neighbours) = self.edges.get(&node_1) else {
            return Err(format!("There is no node with id {} that has any neighbour with id {}!", node_1, node_2));
        };
        Ok(neighbours
            .iter()
            .any(|relationship| relationship.to == node_2))
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
            .or_default()
            .push(relationship.clone());
        relationship.to = node1.id;
        self.edges.entry(node2.id).or_default().push(relationship);
    }

    pub fn add_park_and_ride_to_edge(
        &mut self,
        node_id_1: NodeID,
        node_id_2: NodeID,
    ) -> Result<(), String> {
        match self.add_park_and_ride_to_relationship(node_id_1, node_id_2) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        match self.add_park_and_ride_to_relationship(node_id_2, node_id_1) {
            Ok(_) => Ok(()),
            Err(e) => {
                let mut err_string = String::new();
                match self.remove_park_and_ride_in_relationship(node_id_1, node_id_2) {
                    Ok(_) => (),
                    Err(e) => err_string = e,
                }
                Err(format!("{} and secondly {}", e, err_string))
            }
        }
    }

    fn add_park_and_ride_to_relationship(
        &mut self,
        from_node_id: NodeID,
        to_node_id: NodeID,
    ) -> Result<(), String> {
        self.set_park_and_ride_state_in_relationship(from_node_id, to_node_id, true)
    }

    fn remove_park_and_ride_in_relationship(
        &mut self,
        from_node_id: NodeID,
        to_node_id: NodeID,
    ) -> Result<(), String> {
        self.set_park_and_ride_state_in_relationship(from_node_id, to_node_id, false)
    }

    fn set_park_and_ride_state_in_relationship(
        &mut self,
        from_node_id: NodeID,
        to_node_id: NodeID,
        is_park_and_ride: bool,
    ) -> Result<(), String> {
        match self.are_nodes_neighbours(from_node_id, to_node_id) {
            Ok(n) => {
                if !n {
                    return Err(format!("The node {} is not neighbours with node {} and can therefore not put park and ride between them!", from_node_id, to_node_id));
                }
            }
            Err(e) => return Err(e),
        }
        let Some(neighbours) = self.edges.get_mut(&from_node_id) else {
            return Err(format!("There is no node with id {} that has any neighbours! Therefore we cannot place park and ride!", from_node_id));
        };

        for mut neighbour in neighbours {
            if neighbour.to != to_node_id {
                continue;
            }
            neighbour.is_park_and_ride = is_park_and_ride;
        }

        Ok(())
    }

    fn remove_park_and_ride_from_edge(
        &mut self,
        node_id_1: u8,
        node_id_2: u8,
    ) -> Result<(), String> {
        match self.remove_park_and_ride_in_relationship(node_id_1, node_id_2) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        match self.remove_park_and_ride_in_relationship(node_id_2, node_id_1) {
            Ok(_) => Ok(()),
            Err(e) => {
                let mut err_string = String::new();
                match self.add_park_and_ride_to_edge(node_id_1, node_id_2) {
                    Ok(_) => (),
                    Err(e) => err_string = e,
                }
                Err(format!("{} and secondly {}", e, err_string))
            }
        }
    }

    pub fn cost_to_use_park_and_ride_to_deliver(&self, current_pos_id: NodeID, destination_node_id: NodeID) -> Result<MovementCost, String> {
        match self.nodes.iter().find(|node| node.id == current_pos_id) {
            Some(node) => {
                if !node.is_parking_spot {
                    return Err(format!("The node you are at (node-id: {}) is not a park and ride spot! It's therefore not possible to get a park & ride cost estimate!", current_pos_id));
                }
            },
            None => return Err(format!("The node with id {} does not exist on the map. And can therefore not get a park & ride cost estimate!", current_pos_id)),
        }

        let mut park_and_ride_nodes = Vec::new();
        self.recursively_get_park_and_ride_node_ids_with_cost(current_pos_id, 0, &mut park_and_ride_nodes, &mut Vec::new());

        let (_, cost) = match park_and_ride_nodes.iter().find(|(id, _)| *id == destination_node_id) {
            Some((id, cost)) => (id, cost),
            None => return Err(format!("The destination node id {} is not a park & ride spot connected to the current park & ride spot! It's therefore not possible to get a park & ride cost estimate!", destination_node_id)),
        };

        Ok(*cost)
    }

    fn recursively_get_park_and_ride_node_ids_with_cost(&self, node_id_to_check: NodeID, previous_cost: MovementCost, park_and_ride_nodes: &mut Vec<(NodeID, MovementCost)>, checked_nodes: &mut Vec<NodeID>) {
        if checked_nodes.contains(&node_id_to_check) {
            return;
        }
        checked_nodes.push(node_id_to_check);

        let Some(neighbour_relationships) = self.edges.get(&node_id_to_check) else {
            return;
        };

        for neighbour_relationship in neighbour_relationships {
            if neighbour_relationship.is_park_and_ride {
                park_and_ride_nodes.push((neighbour_relationship.to, previous_cost + neighbour_relationship.movement_cost));
                self.recursively_get_park_and_ride_node_ids_with_cost(neighbour_relationship.to, previous_cost + neighbour_relationship.movement_cost, park_and_ride_nodes, checked_nodes);
            }
        }
    }
}

impl CostTuple {
    pub const fn new(neighbourhood: Neighbourhood, traffic: Traffic) -> Self {
        Self {
            neighbourhood,
            traffic,
        }
    }
}

impl SituationCard {
    #[must_use]
    pub const fn new(
        card_id: SituationCardID,
        title: String,
        description: String,
        goal: String,
        costs: Vec<CostTuple>,
        objective_cards: Vec<PlayerObjectiveCard>,
    ) -> Self {
        Self {
            card_id,
            title,
            description,
            goal,
            costs,
            objective_cards,
        }
    }
}

impl SituationCardList {
    #[must_use]
    pub const fn new(situation_cards: Vec<SituationCard>) -> Self {
        Self { situation_cards }
    }

    pub fn get_default_situation_card_by_id(id: SituationCardID) -> Result<SituationCard, String> {
        let situation_cards = crate::situation_card_list::situation_card_list_wrapper();
        situation_cards
            .situation_cards
            .iter()
            .find(|card| card.card_id == id)
            .map_or_else(
                || Err(format!("There was no code with the ID: {}", id)),
                |card| Ok(card.clone()),
            )
    }
}

impl PlayerObjectiveCard {
    pub fn new(
        start_node_id: NodeID,
        pick_up_node_id: NodeID,
        drop_off_node_id: NodeID,
        special_vehicle_types: Vec<VehicleType>,
    ) -> Self {
        Self {
            start_node_id,
            pick_up_node_id,
            drop_off_node_id,
            special_vehicle_types,
            picked_package_up: false,
            dropped_package_off: false,
        }
    }
}

impl Traffic {
    pub const fn get_movement_cost(&self) -> MovementCost {
        match self {
            Self::LevelOne => 0,
            Self::LevelTwo => 0,
            Self::LevelThree => 1,
            Self::LevelFour => 2,
            Self::LevelFive => 4,
        }
    }

    pub const fn increased(&self) -> Self {
        match self {
            Self::LevelOne => Self::LevelTwo,
            Self::LevelTwo => Self::LevelThree,
            Self::LevelThree => Self::LevelFour,
            Self::LevelFour => Self::LevelFive,
            Self::LevelFive => Self::LevelFive,
        }
    }
}

impl VehicleType {
    pub const fn times_to_increase_traffic_when_access(&self) -> usize {
        match self {
            Self::Electric => 2,
            Self::Buss => 1,
            Self::Emergency => 0,
            Self::Industrial => 1,
            Self::Normal => 1,
            Self::Geolocation => 1,
        }
    }
}
