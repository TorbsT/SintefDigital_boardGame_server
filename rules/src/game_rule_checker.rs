use std::ops::ControlFlow;

use game_core::{
    game_data::{DistrictModifierType, GameState, InGameID, PlayerInput, PlayerInputType, EdgeRestriction, NeighbourRelationship, RestrictionType, NodeID, Player, Neighbourhood},
    rule_checker::{ErrorData, RuleChecker},
};

type RuleFn = Box<dyn Fn(&GameState, &PlayerInput) -> ValidationResponse<String> + Send + Sync>;

struct Rule {
    pub related_inputs: Vec<PlayerInputType>,
    pub rule_fn: RuleFn,
}

pub struct GameRuleChecker {
    rules: Vec<Rule>,
}

enum ValidationResponse<T> {
    Valid,
    Invalid(T),
}

impl RuleChecker for GameRuleChecker {
    fn is_input_valid(&self, game: &GameState, player_input: &PlayerInput) -> Option<ErrorData> {
        let mut error_str = "Invalid input!".to_string();
        let foreach_status = &self.rules.iter().try_for_each(|rule| {
            if rule.related_inputs.iter().all(|input_type| {
                input_type != &player_input.input_type && input_type != &PlayerInputType::All
            }) {
                return ControlFlow::Continue(());
            }

            match (rule.rule_fn)(game, player_input) {
                ValidationResponse::Valid => ControlFlow::Continue(()),
                ValidationResponse::Invalid(e) => {
                    error_str = e;
                    ControlFlow::Break(false)
                }
            }
        });
        if foreach_status.eq(&ControlFlow::Break(false)) {
            return Some(error_str);
        }
        None
    }
}

impl Default for GameRuleChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRuleChecker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Self::get_rules(),
        }
    }

    fn get_rules() -> Vec<Rule> {
        let game_started = Rule {
            related_inputs: vec![
                PlayerInputType::Movement,
                PlayerInputType::ModifyDistrict,
                PlayerInputType::NextTurn,
                PlayerInputType::UndoAction,
            ],
            rule_fn: Box::new(has_game_started),
        };
        let players_turn = Rule {
            related_inputs: vec![PlayerInputType::All],
            rule_fn: Box::new(is_players_turn),
        };
        let orchestrator_check = Rule {
            related_inputs: vec![
                PlayerInputType::StartGame,
                PlayerInputType::ModifyEdgeRestrictions,
                PlayerInputType::ModifyDistrict,
            ],
            rule_fn: Box::new(is_orchestrator),
        };
        let player_has_position = Rule {
            related_inputs: vec![PlayerInputType::Movement],
            rule_fn: Box::new(has_position),
        };
        let toggle_bus = Rule {
            related_inputs: vec![PlayerInputType::SetPlayerBusBool],
            rule_fn: Box::new(can_toggle_bus),
        };
        let toggle_train = Rule {
            related_inputs: vec![PlayerInputType::SetPlayerTrainBool],
            rule_fn: Box::new(can_toggle_train),
        };
        let next_to_node = Rule {
            related_inputs: vec![PlayerInputType::Movement],
            rule_fn: Box::new(next_node_is_neighbour),
        };
        let enough_moves = Rule {
            related_inputs: vec![PlayerInputType::Movement],
            rule_fn: Box::new(has_enough_moves),
        };
        let move_to_node = Rule {
            related_inputs: vec![PlayerInputType::Movement],
            rule_fn: Box::new(can_move_to_node),
        };
        let can_modify_edge_restriction = Rule {
            related_inputs: vec![PlayerInputType::ModifyEdgeRestrictions],
            rule_fn: Box::new(is_edge_modification_action_valid),
        };

        let rules = vec![
            game_started,
            players_turn,
            orchestrator_check,
            player_has_position,
            toggle_bus,
            toggle_train,
            next_to_node,
            enough_moves,
            move_to_node,
            can_modify_edge_restriction,
        ];
        rules
    }
}

// ================== MACROS ====================
macro_rules! get_player_or_return_invalid_response {
    ($game:expr, $player_input:expr) => {{
        let player_result = $game.get_player_with_unique_id($player_input.player_id);
        let player = match player_result {
            Ok(p) => p,
            Err(e) => return ValidationResponse::Invalid(e.to_string()),
        };
        player.clone()
    }};
}

macro_rules! get_player_position_id_or_return_invalid_response {
    ($player:expr) => {{
        match $player.position_node_id {
            Some(id) => id,
            None => return ValidationResponse::Invalid("The player does not have a position and can therefore not check if it's a valid action!".to_string()),
        }
    }};
}

// ==================== RULES ====================

fn has_game_started(game: &GameState, _player_input: &PlayerInput) -> ValidationResponse<String> {
    match game.is_lobby {
        true => ValidationResponse::Invalid("The game has not started yet!".to_string()),
        false => ValidationResponse::Valid,
    }
}

fn has_enough_moves(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);

    if player.remaining_moves == 0 {
        return ValidationResponse::Invalid("The player has no remaining moves!".to_string());
    }

    let Some(related_node_id) = player_input.related_node_id else {
        return ValidationResponse::Invalid("There was no node to get cost to!".to_string());
    };

    let mut game_clone = game.clone();

    match game_clone.move_player_with_id(player_input.player_id, related_node_id) {
        Ok(_) => (),
        Err(e) => return ValidationResponse::Invalid(e),
    }

    has_non_negative_amount_of_moves_left(game, player_input)
}

fn has_non_negative_amount_of_moves_left(
    game: &GameState,
    player_input: &PlayerInput,
) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);

    if player.remaining_moves < 0 {
        return ValidationResponse::Invalid(
            format!("The player does not have enough remaining moves! The player would have {} remaining moves!", player.remaining_moves),
        );
    }

    ValidationResponse::Valid
}

fn can_enter_district(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);

    let district_modifiers = &game.district_modifiers;

    let player_objective_card = match player.objective_card {
        Some(objective_card) => objective_card,
        None => {
            return ValidationResponse::Invalid(
                "Error: Player does not have an objective card".to_string(),
            )
        }
    };

    let neighbours = match player.position_node_id {
        Some(pos) => match game.map.get_neighbour_relationships_of_node_with_id(pos) {
            Some(vec) => vec,
            None => {
                return ValidationResponse::Invalid(format!(
                    "Error: Node with ID {} does not exist",
                    pos
                ))
            }
        },
        None => {
            return ValidationResponse::Invalid(
                "Error: Player does not have a valid position and can therefore not move"
                    .to_string(),
            )
        }
    };

    let Some(to_node_id) = player_input.related_node_id else {
        return ValidationResponse::Invalid("Error: Related node ID does not exist in player input and has to be set for player movement".to_string());
    };
    let Some(neighbour_relationship) = neighbours.iter().find(|neighbour| neighbour.to == to_node_id) else {
        return ValidationResponse::Invalid("Error: There is no neighbouring node with the ID given".to_string());
    };

    let mut district_has_modifier = false;
    for dm in district_modifiers {
        if dm.district != neighbour_relationship.neighbourhood
            || dm.modifier != DistrictModifierType::Access
        {
            continue;
        }
        let Some(vehicle_type) = dm.vehicle_type else {
            return ValidationResponse::Invalid("Error: There was no vehicle for access modifier".to_string());
        };
        district_has_modifier = true;
        if player_objective_card
            .special_vehicle_types
            .contains(&vehicle_type)
        {
            return ValidationResponse::Valid;
        }
    }

    if !district_has_modifier {
        return ValidationResponse::Valid;
    }
    ValidationResponse::Invalid(
        "Invalid move: Player does not have required vehicle type to access this district"
            .to_string(),
    )
}

pub fn player_has_objective_in_district(game: GameState, player: Player, district: Neighbourhood) -> bool {
    let Some(objectivecard) = player.objective_card else {
        return false;
    };
    let Some(player_pickup_node_neighbours) = game.map.get_neighbour_relationships_of_node_with_id(objectivecard.pick_up_node_id) else {
        return false;
    };
    let Some(player_drop_off_node_neighbours) = game.map.get_neighbour_relationships_of_node_with_id(objectivecard.drop_off_node_id) else {
        return false;
    };
    node_is_in_district(player_pickup_node_neighbours, district) || node_is_in_district(player_drop_off_node_neighbours, district)
}

pub fn node_is_in_district (neighbour_list: Vec<NeighbourRelationship>, district: Neighbourhood) -> bool {
    let mut node_is_in_district = false;
    neighbour_list.into_iter().for_each(|edge|{
        if edge.neighbourhood == district {
            node_is_in_district = true;
        }
    });
    node_is_in_district
}

fn has_position(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    match game.get_player_with_unique_id(player_input.player_id) {
        Ok(p) => {
            if p.position_node_id.is_none() {
                return ValidationResponse::Invalid(
                    "The player does not have a position!".to_string(),
                );
            }
            ValidationResponse::Valid
        }
        Err(e) => ValidationResponse::Invalid(e.to_string()),
    }
}

fn next_node_is_neighbour(
    game: &GameState,
    player_input: &PlayerInput,
) -> ValidationResponse<String> {
    match game.get_player_with_unique_id(player_input.player_id) {
        Ok(p) => {
            match p.position_node_id {
                Some(node_id) => {
                    let Some(related_node_id) = player_input.related_node_id else {
                        return ValidationResponse::Invalid("There was node to check if it's a neighbour!".to_string());
                    };
                    let are_neighbours =
                        match game.map.are_nodes_neighbours(node_id, related_node_id) {
                            Ok(b) => b,
                            Err(e) => return ValidationResponse::Invalid(e),
                        };
                    if !are_neighbours {
                        return ValidationResponse::Invalid(format!(
                            "The node {related_node_id} is not a neighbour of the player's position!",
                        ));
                    }
                }
                None => {
                    return ValidationResponse::Invalid(
                        "The player does not have a position!".to_string(),
                    )
                }
            }
            ValidationResponse::Valid
        }
        Err(e) => ValidationResponse::Invalid(e.to_string()),
    }
}

fn is_players_turn(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    if game.is_lobby || player_input.input_type == PlayerInputType::LeaveGame {
        return ValidationResponse::Valid;
    }

    let player = get_player_or_return_invalid_response!(game, player_input);

    if game.current_players_turn != player.in_game_id {
        return ValidationResponse::Invalid("It's not the current players turn".to_string());
    }

    ValidationResponse::Valid
}

fn is_orchestrator(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);
    if player.in_game_id != InGameID::Orchestrator {
        return ValidationResponse::Invalid(
            "The player is not the orchestrator of the game!".to_string(),
        );
    }

    ValidationResponse::Valid
}

fn is_edge_modification_action_valid(
    game: &GameState,
    player_input: &PlayerInput,
) -> ValidationResponse<String> {
    let Some(edge_mod) = player_input.edge_modifier.clone() else {
        return ValidationResponse::Invalid("There was no modifier on the edge modifier player input, and can therefore not check the input further!".to_string());
    };

    let Some(neighbours_one) = game.map.get_neighbour_relationships_of_node_with_id(edge_mod.node_one) else {
        return ValidationResponse::Invalid(format!("The node {} does not have neighbours and can therefore not have restrictions!", edge_mod.node_one));
    };

    let Some(neighbours_two) = game.map.get_neighbour_relationships_of_node_with_id(edge_mod.node_two) else {
        return ValidationResponse::Invalid(format!("The node {} does not have neighbours and can therefore not have restrictions!", edge_mod.node_one));
    };

    match edge_mod.edge_restriction {
        game_core::game_data::RestrictionType::ParkAndRide => can_modify_park_and_ride(game, &edge_mod, &neighbours_one, &neighbours_two),
        _ => default_can_modify_edge_restriction(&edge_mod, &neighbours_one, edge_mod.node_two),
    }

}

fn default_can_modify_edge_restriction(edge_mod: &EdgeRestriction, neighbours_one: &[NeighbourRelationship], node_two_id: NodeID) -> ValidationResponse<String> {
    if neighbours_one.iter().any(|relationship| relationship.to == node_two_id && relationship.restriction == Some(edge_mod.edge_restriction)) {
        if edge_mod.delete {
            return ValidationResponse::Valid;
        }
        ValidationResponse::Invalid(format!("The edge restriction {:?} already exists on the edge between node {} and node {}!", edge_mod.edge_restriction, edge_mod.node_one, edge_mod.node_two)) 
    } else {
        ValidationResponse::Invalid(format!("The edge restriction {:?} does not exist on the edge between node {} and node {}!", edge_mod.edge_restriction, edge_mod.node_one, edge_mod.node_two))
    }
}

fn can_modify_park_and_ride(game: &GameState, park_and_ride_mod: &EdgeRestriction, neighbours_one: &[NeighbourRelationship], neighbours_two: &[NeighbourRelationship]) -> ValidationResponse<String> {
    if park_and_ride_mod.delete {
        if neighbours_one
            .iter()
            .filter(|neighbour| neighbour.restriction == Some(RestrictionType::ParkAndRide))
            .count()
            < 2
            || neighbours_two
                .iter()
                .filter(|neighbour| neighbour.restriction == Some(RestrictionType::ParkAndRide))
                .count()
                < 2
        {
            return ValidationResponse::Valid;
        }
        return ValidationResponse::Invalid("It's not possible to delete a park & ride edge that is connected to more than one other park & ride edge!".to_string());
    }

    let node_one = match game.map.get_node_by_id(park_and_ride_mod.node_one) {
        Ok(n) => n,
        Err(e) => {
            return ValidationResponse::Invalid(
                e + " and can therefore not check wether the park & ride can be placed here!",
            )
        }
    };

    let node_two = match game.map.get_node_by_id(park_and_ride_mod.node_two) {
        Ok(n) => n,
        Err(e) => {
            return ValidationResponse::Invalid(
                e + " and can therefore not check wether the park & ride can be placed here!",
            )
        }
    };

    if node_one.is_parking_spot || node_two.is_parking_spot {
        return ValidationResponse::Valid;
    }

    if neighbours_one
        .iter()
        .filter(|neighbour| neighbour.restriction == Some(RestrictionType::ParkAndRide))
        .count()
        > 0
        || neighbours_two
            .iter()
            .filter(|neighbour| neighbour.restriction == Some(RestrictionType::ParkAndRide))
            .count()
            > 0
    {
        return ValidationResponse::Valid;
    }

    ValidationResponse::Invalid(format!("Cannot place park & ride on the edge between node with ids {} and {} because there is no adjacent parking spots or park and ride edges!", park_and_ride_mod.node_one, park_and_ride_mod.node_two))
}

fn can_move_to_node(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);
    
    let player_pos = get_player_position_id_or_return_invalid_response!(player);

    let Some(to_node_id) = player_input.related_node_id else {
        return ValidationResponse::Invalid("There is no related node to the movement input. There needs to be a node if a players should move!".to_string());
    };

    let Some(neighbours) = game.map.get_neighbour_relationships_of_node_with_id(player_pos) else {
        return ValidationResponse::Invalid(format!("The node {} does not have neighbours and can therefore not have park and ride!", player_pos));
    };

    if player.is_bus {
        if neighbours
            .iter()
            .any(|neighbour| neighbour.restriction == Some(RestrictionType::ParkAndRide) && neighbour.to == to_node_id)
        {
            return ValidationResponse::Valid;
        }
        return ValidationResponse::Invalid(
            format!("The player cannot move here because the node (with id {}) is not a neighbouring node connected with a park & ride edge!", to_node_id),
        );
    }

    if player.is_train {
        if neighbours
            .iter()
            .any(|neighbour| neighbour.is_connected_through_rail && neighbour.to == to_node_id)
        {
            return ValidationResponse::Valid;
        }
        return ValidationResponse::Invalid(
            format!("The player cannot move here because the node (with id {}) is not a neighbouring node connected through the railway!", to_node_id),
        );
    }
    
    let Some(neighbour_relationship) = neighbours.iter().find(|neighbour| neighbour.to == to_node_id) else {
        return ValidationResponse::Invalid(format!("The node {} is not a neighbour of the node {} and can therefore not be moved to!", to_node_id, player_pos));
    };

    if let Some(restriction) = neighbour_relationship.restriction {
        let Some(objective_card) = player.objective_card else {
            return ValidationResponse::Invalid(format!("The player {} does not have an objective card and we can therefore not check if the player has access to the given zone!", player.name));
        };

        if !objective_card.special_vehicle_types.contains(&restriction) {
            return ValidationResponse::Invalid(format!("The player {} does not have access to the edge {:?} and can therefore not move to the node {}!", player.name, restriction, to_node_id));
        }

        return ValidationResponse::Valid;
    }

    match can_enter_district(game, player_input) {
        ValidationResponse::Valid => (),
        ValidationResponse::Invalid(e) => return ValidationResponse::Invalid(e),
    }

    if neighbours
        .iter()
        .any(|neighbour| neighbour.restriction == Some(RestrictionType::ParkAndRide) && neighbour.to == to_node_id)
    {
        return ValidationResponse::Invalid(
            "The player cannot move here because it's a park & ride edge!".to_string(),
        );
    }

    ValidationResponse::Valid
}

fn can_toggle_bus(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);
    
    let Some(_) = player_input.related_bool else {
        return ValidationResponse::Invalid("Could not check if you can toggle bus because the related bool was not set. It's needed for so that we can know if you want to stop being a bus or change to a bus!".to_string());
    };
    
    if player.is_train {
        return ValidationResponse::Invalid("You cannot toggle bus if you are a train!".to_string());
    }

    let player_pos = get_player_position_id_or_return_invalid_response!(player);
    let node = match game.map.get_node_by_id(player_pos) {
        Ok(n) => n,
        Err(e) => {
            return ValidationResponse::Invalid(
                e + " and can therefore not check wether the player can toggle bus!",
            )
        }
    };

    if !node.is_parking_spot {
        return ValidationResponse::Invalid(
            "You cannot toggle bus if you are not on a parking spot!".to_string(),
        );
    }

    ValidationResponse::Valid
}

fn can_toggle_train(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player = get_player_or_return_invalid_response!(game, player_input);
    
    let Some(_) = player_input.related_bool else {
        return ValidationResponse::Invalid("Could not check if you can toggle train because the related bool was not set. It's needed for so that we can know if you want to stop being a train or change to a train!".to_string());
    };
    
    if player.is_bus {
        return ValidationResponse::Invalid("You cannot toggle train if you are a bus!".to_string());
    }

    let player_pos = get_player_position_id_or_return_invalid_response!(player);
    let node = match game.map.get_node_by_id(player_pos) {
        Ok(n) => n,
        Err(e) => {
            return ValidationResponse::Invalid(
                e + " and can therefore not check wether the player can toggle train!",
            )
        }
    };

    if !node.is_connected_to_rail {
        return ValidationResponse::Invalid(
            "You cannot toggle train if you are not on a train station spot!".to_string(),
        );
    }

    ValidationResponse::Valid
}
// TODO: Check if a player is on the destination node before letting them pressing next turn
