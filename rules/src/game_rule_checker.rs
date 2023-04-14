use std::ops::ControlFlow;

use game_core::{
    game_data::{GameState, NodeMap, PlayerInput, PlayerInputType},
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
        let players_turn = Rule {
            related_inputs: vec![PlayerInputType::All],
            rule_fn: Box::new(is_players_turn),
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

        let rules = vec![
            players_turn,
            player_has_position,
            next_to_node,
            enough_moves,
        ];
        rules
    }
}

// ==================== RULES ====================

fn has_enough_moves(game: &GameState, player_input: &PlayerInput) -> ValidationResponse<String> {
    let player_result = game.get_player_with_unique_id(player_input.player_id);
    let player = match player_result {
        Ok(p) => p,
        Err(e) => return ValidationResponse::Invalid(e.to_string()),
    };

    if player.remaining_moves == 0 {
        return ValidationResponse::Invalid("The player has no remaining moves!".to_string());
    }

    let Some(position_node_id) = player.position_node_id else {
        return ValidationResponse::Invalid(format!("Player {} has no position!", player.unique_id));
    };

    let map = NodeMap::new();
    let current_node = match map.get_node_by_id(position_node_id) {
        Ok(node) => node,
        Err(e) => return ValidationResponse::Invalid(e),
    };

    let Some(related_node_id) = player_input.related_node_id else {
        return ValidationResponse::Invalid("There was no node to get cost to!".to_string());
    };

    let cost = match current_node.get_movement_cost_to_neighbour_with_id(related_node_id) {
        Ok(cost) => cost,
        Err(e) => return ValidationResponse::Invalid(e),
    };

    if player.remaining_moves < cost {
        return ValidationResponse::Invalid(
            "The player has not enough remaining moves!".to_string(),
        );
    }

    let related_node = match map.get_node_by_id(related_node_id) {
        Ok(node) => node,
        Err(e) => return ValidationResponse::Invalid(e),
    };

    ValidationResponse::Valid
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
                    let map = NodeMap::new();
                    match map.get_node_by_id(node_id) {
                        Ok(node) => {
                            let Some(related_node_id) = player_input.related_node_id else {
                                return ValidationResponse::Invalid("There was node to check if it's a neighbour!".to_string());
                            };
                            if !node.has_neighbour_with_id(related_node_id) {
                                return ValidationResponse::Invalid(format!(
                                    "The node {related_node_id} is not a neighbour of the player's position!",
                                ));
                            }
                        }
                        Err(e) => return ValidationResponse::Invalid(e),
                    };
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

    let player = match game.get_player_with_unique_id(player_input.player_id) {
        Ok(p) => p,
        Err(e) => return ValidationResponse::Invalid(e.to_string()),
    };

    if game.current_players_turn != player.in_game_id {
        return ValidationResponse::Invalid("It's not the current players turn".to_string());
    }

    ValidationResponse::Valid
}

// TODO: Check if a player is on the destination node before letting them pressing next turn
