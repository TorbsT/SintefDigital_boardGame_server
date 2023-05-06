use std::ops::ControlFlow;

use game_core::{
    game_data::{DistrictModifierType, GameState, InGameID, PlayerInput, PlayerInputType},
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
                PlayerInputType::ModifyParkAndRide,
                PlayerInputType::ModifyDistrict,
            ],
            rule_fn: Box::new(is_orchestrator),
        };

        let player_has_position = Rule {
            related_inputs: vec![PlayerInputType::Movement],
            rule_fn: Box::new(has_position),
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
        let can_modify_park_and_ride = Rule {
            related_inputs: vec![PlayerInputType::ModifyParkAndRide],
            rule_fn: Box::new(is_park_and_ride_action_valid),
        };

        let rules = vec![
            game_started,
            players_turn,
            orchestrator_check,
            player_has_position,
            next_to_node,
            enough_moves,
            move_to_node,
            can_modify_park_and_ride,
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
    if game.is_lobby {
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

fn is_park_and_ride_action_valid(
    game: &GameState,
    player_input: &PlayerInput,
) -> ValidationResponse<String> {
    let Some(park_and_ride_mod) = player_input.park_and_ride_modifier.clone() else {
        return ValidationResponse::Invalid("There was no park_and_ride modifier on the park and ride player input, and can therefore not check the input further!".to_string());
    };

    let Some(neighbours_one) = game.map.get_neighbour_relationships_of_node_with_id(park_and_ride_mod.node_one) else {
        return ValidationResponse::Invalid(format!("The node {} does not have neighbours and can therefore not have park and ride!", park_and_ride_mod.node_one));
    };

    let Some(neighbours_two) = game.map.get_neighbour_relationships_of_node_with_id(park_and_ride_mod.node_two) else {
        return ValidationResponse::Invalid(format!("The node {} does not have neighbours and can therefore not have park and ride!", park_and_ride_mod.node_one));
    };

    if park_and_ride_mod.delete {
        if neighbours_one
            .iter()
            .filter(|neighbour| neighbour.is_park_and_ride)
            .count()
            < 2
            || neighbours_two
                .iter()
                .filter(|neighbour| neighbour.is_park_and_ride)
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
        .filter(|neighbour| neighbour.is_park_and_ride)
        .count()
        > 0
        || neighbours_two
            .iter()
            .filter(|neighbour| neighbour.is_park_and_ride)
            .count()
            > 0
    {
        return ValidationResponse::Valid;
    }

    ValidationResponse::Invalid(format!("Cannot place park & ride on the edge between node with ids {} and {} because there is no adjacent parking spots or park and ride edges!", park_and_ride_mod.node_one, park_and_ride_mod.node_two))
}

fn can_move_to_node(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    match can_enter_district(game, player_input) {
        ValidationResponse::Valid => (),
        ValidationResponse::Invalid(e) => return ValidationResponse::Invalid(e),
    }

    let player = get_player_or_return_invalid_response!(game, player_input);

    let player_pos = get_player_position_id_or_return_invalid_response!(player);

    let Some(to_node_id) = player_input.related_node_id else {
        return ValidationResponse::Invalid("There is no related node to the movement input. There needs to be a node if a players should move!".to_string());
    };

    let Some(neighbours) = game.map.get_neighbour_relationships_of_node_with_id(player_pos) else {
        return ValidationResponse::Invalid(format!("The node {} does not have neighbours and can therefore not have park and ride!", player_pos));
    };

    if neighbours
        .iter()
        .any(|neighbour| neighbour.is_park_and_ride && neighbour.to == to_node_id)
    {
        return ValidationResponse::Invalid(
            "The player cannot move here because it's a park & ride edge!".to_string(),
        );
    }

    ValidationResponse::Valid
}

// TODO: Check if a player is on the destination node before letting them pressing next turn
