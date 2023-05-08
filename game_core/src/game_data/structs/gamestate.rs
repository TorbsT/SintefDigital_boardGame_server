use std::{cmp, mem};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{game_data::{custom_types::{GameID, NodeID, PlayerID, MovementCost, MovementValue}, enums::{in_game_id::InGameID, district::District, restriction_type::RestrictionType, district_modifier_type::DistrictModifierType, traffic::Traffic}, constants::{MAX_PLAYER_COUNT, START_MOVEMENT_AMOUNT, MAX_ACCESS_MODIFIER_COUNT, MAX_PRIORITY_MODIFIER_COUNT, MAX_TOLL_MODIFIER_COUNT}}, situation_card_list::situation_card_list};

use super::{player::Player, player_input::PlayerInput, situation_card::SituationCard, edge_restriction::EdgeRestriction, node_map::NodeMap, neighbour_relationship::NeighbourRelationship, district_modifier::DistrictModifier};

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
    pub accessed_districts: Vec<District>,
    #[serde(skip)]
    pub map: NodeMap,
    pub situation_card: Option<SituationCard>,
    pub edge_restrictions: Vec<EdgeRestriction>,
    pub legal_nodes: Vec<NodeID>,
}

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
            edge_restrictions: Vec::new(),
            legal_nodes: Vec::new(),
        }
    }

    pub fn set_player_bus_bool(&mut self, player_id: PlayerID, boolean: bool) {
        for player in self.players.iter_mut() {
            if player.unique_id != player_id {
                continue;
            }
            player.is_bus = boolean;
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

    fn node_is_in_district (neighbour_list: Vec<NeighbourRelationship>, district: District) -> bool {
        let mut node_is_in_district = false;
        neighbour_list.into_iter().for_each(|edge|{
            if edge.neighbourhood == district {
                node_is_in_district = true;
            }
        });
        node_is_in_district
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

            let current_node = match self.map.get_node_by_id(current_node_id) {
                Ok(n) => n,
                Err(e) => return Err(e),
            };

            let to_node = match self.map.get_node_by_id(to_node_id) {
                Ok(n) => n,
                Err(e) => return Err(e),
            };

            let Some(neighbours) = self.map.get_neighbour_relationships_of_node_with_id(current_node_id) else {
                return Err(format!("There was no node with id {}!", current_node_id));
            };

            let Some(neighbour_relationship) = neighbours.iter().find(|relationship| relationship.to == to_node_id) else {
                return Err(format!("The node you are trying to go to is not a neighbour. From node with id {} to {}", current_node_id, to_node_id));
            }; // TODO This check should be done in rule checker!
            if neighbour_relationship.blocked {
                return Err(format!("The road from id {} to id {} has blocked traffic going in that direciton", current_node_id, to_node_id));
            }

            // TODO: This check should be done in the rule checker!
            if to_node.is_connected_to_rail && current_node.is_connected_to_rail {
                if neighbour_relationship.is_connected_through_rail {
                    Self::move_player_to_node(player, to_node_id, 1);
                    return Ok(());
                }
                return Err(format!("The node you are trying to go to (with id {}) is not a neighbouring train station and you can therefore not move to it as a train!", to_node_id));
            }

            if player.is_bus {
                let Some(edge_restriction) = neighbour_relationship.restriction else {
                    return Err(format!("The node (with id {}) you are trying to go to does not have a restriction and you can therefore not move there as a bus!", to_node_id));
                };

                if edge_restriction != RestrictionType::ParkAndRide {
                    return Err(format!("The node (with id {}) you are trying to go to is not a part of the park & ride roads and you can therefore not move there as a bus!", to_node_id));
                }
                
                Self::move_player_to_node(player, to_node_id, 1);
                return Ok(());
            }

            if let Some(restriction) = neighbour_relationship.restriction {
                if restriction == RestrictionType::ParkAndRide {
                    return Err(format!("The node (with id {}) you are trying to go to is a part of the park & ride roads and you can therefore not move there unless you are a buss!", to_node_id));
                }
                Self::move_player_to_node(player, to_node_id, 1);
                return Ok(());
            }

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
                        if modifier.modifier == DistrictModifierType::Toll {
                            continue; //TODO: Implement toll
                        }

                        let player_has_objective_in_district = Self::player_has_objective_in_district(&self.map, player, modifier.district);
                        
                        let Some(restriction_vehicle_type) = modifier.vehicle_type else {
                            return Err("The vehicle type can not be determined, and bonus moves can not be applied".to_string());
                        };

                        if modifier.district != neighbour_relationship.neighbourhood {
                            continue;
                        }

                        if restriction_vehicle_type == RestrictionType::Destination && player_has_objective_in_district {
                            if let Some(movement_value) = modifier.associated_movement_value {
                                bonus_moves = cmp::max(bonus_moves, movement_value);
                            }
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
            }
            player.remaining_moves -= neighbour_relationship.movement_cost;
            player.position_node_id = Some(to_node_id);
            return Ok(());
        }
        Err("There were no players in this game that match the player to update".to_string())
    }

    pub fn player_has_objective_in_district(map: &NodeMap, player: &Player, district: District) -> bool {
        let Some(objectivecard) = &player.objective_card else {
            return false;
        };
        let Some(player_pickup_node_neighbours) = map.get_neighbour_relationships_of_node_with_id(objectivecard.pick_up_node_id) else {
            return false;
        };
        let Some(player_drop_off_node_neighbours) = map.get_neighbour_relationships_of_node_with_id(objectivecard.drop_off_node_id) else {
            return false;
        };
        Self::node_is_in_district(player_pickup_node_neighbours, district) || Self::node_is_in_district(player_drop_off_node_neighbours, district)
    }

    fn move_player_to_node(player: &mut Player, to_node_id: NodeID, cost: MovementCost) {
        player.remaining_moves -= cost;
        player.position_node_id = Some(to_node_id);
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
                self.is_lobby = true;
            }
            if counter >= 1000 {
                next_player_turn = InGameID::Orchestrator;
                break;
            }
            counter += 1;
        }
        self.accessed_districts.clear();
        self.current_players_turn = next_player_turn;
    }

    pub const fn get_starting_player_movement_value() -> MovementValue {
        START_MOVEMENT_AMOUNT
    }

    pub fn assign_random_objective_card_to_players(&mut self) -> Result<(), String> {
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

    pub fn start_game(&mut self) -> Result<(), String> {
        let mut can_start_game = false;
        let mut errormessage =
            String::from("Unable to start game because lobby does not have an orchestrator");
        self.reset_player_in_game_data();
        self.edge_restrictions.clear();
        self.district_modifiers.clear();
        match self.update_node_map_with_situation_card() {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
        for player in self.players.clone() {
            if player.in_game_id == InGameID::Undecided {
                errormessage = format!("Unable to start game because player with id {} and name {} is neither player, nor orchestrator (Undecided)", player.unique_id, player.name);
                return Err(errormessage);
            }
        }
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
                match self.assign_random_objective_card_to_players() {
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

    pub fn reset_player_in_game_data(&mut self) {
        for player in self.players.iter_mut() {
            player.position_node_id = None;
            player.remaining_moves = Self::get_starting_player_movement_value();
            player.objective_card = None;
            player.is_bus = false;
        }
    }

    pub fn update_node_map_with_situation_card(&mut self) -> Result<(), String> {
        self.map.reset();
        match &self.situation_card {
            Some(card) => {
                self.map.update_neighbourhood_cost(card);
                match card.card_id {
                    0 => {
                        return Err("Error: Situation card with ID 0 does not exist".to_string());
                    },
                    1 => {},
                    2 => {},
                    3 => {},
                    4 => {
                        match self.add_edge_restriction(&EdgeRestriction { node_one: 19, node_two: 20, edge_restriction: RestrictionType::OneWay, delete: false }, false) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    },
                    5 => {
                        match self.map.toggle_rail_connection_on_node_with_id(24) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        };
                        match self.map.toggle_rail_connection_on_node_with_id(27) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    },
                    _ => {
                        return Err("Error: Situation card with IDs 6 and up do not exist".to_string());
                    },
                }
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
        self.map.update_neighbourhood_cost(&situation_card);
        self.situation_card = Some(situation_card);


        Ok(())
    }

    pub fn add_edge_restriction(
        &mut self,
        edge_restriction: &EdgeRestriction,
        modifiable: bool,
    ) -> Result<(), String> {
        match self.map.set_restriction_on_edge(edge_restriction, modifiable) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        self.edge_restrictions
            .push(edge_restriction.clone());
        Ok(())
    }

    pub fn remove_restriction_from_edge(
        &mut self,
        edge_restriction: &EdgeRestriction,
    ) -> Result<(), String> {
        match self
            .map
            .remove_restriction_from_edge(edge_restriction)
        {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        self.edge_restrictions.retain(|nodes| {
            !((nodes.node_one == edge_restriction.node_one && nodes.node_two == edge_restriction.node_two)
                || (nodes.node_one == edge_restriction.node_two && nodes.node_two == edge_restriction.node_one))
        });
        Ok(())
    }
}