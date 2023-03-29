use std::ops::ControlFlow;

use game_core::{
    game_data::{GameState, NodeMap, PlayerInput, PlayerInputType},
    rule_checker::{ErrorData, RuleChecker},
};

type RuleFn =
    Box<dyn Fn(&GameState, &PlayerInput) -> Result<ValidationResponse, String> + Send + Sync>;

struct Rule {
    pub related_input: PlayerInputType,
    pub rule_fn: RuleFn,
}

pub struct GameRuleChecker {
    rules: Vec<Rule>,
}

struct ValidationResponse {
    pub is_valid: bool,
    pub reason: Option<String>,
}

impl RuleChecker for GameRuleChecker {
    fn is_input_valid(&self, game: &GameState, player_input: &PlayerInput) -> Option<ErrorData> {
        let mut error_str = "Invalid input!".to_string();
        let foreach_status = &self.rules.iter().try_for_each(|rule| {
            if rule.related_input != player_input.input_type {
                return ControlFlow::Continue(());
            }

            match (rule.rule_fn)(game, player_input) {
                Ok(response) => {
                    if !response.is_valid {
                        match response.reason {
                            Some(message) => error_str = message,
                            None => error_str = String::from(
                                "Failed to get reason for invalid input, but the move was invalid!",
                            ),
                        }
                        return ControlFlow::Break(false);
                    }
                    ControlFlow::Continue(())
                }
                Err(e) => {
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

impl ValidationResponse {
    #[must_use]
    pub const fn new(is_valid: bool, reason: Option<String>) -> Self {
        Self { is_valid, reason }
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
        let player_has_position = Rule {
            related_input: PlayerInputType::Movement,
            rule_fn: Box::new(has_position),
        };
        let next_to_node = Rule {
            related_input: PlayerInputType::Movement,
            rule_fn: Box::new(next_node_is_neighbour),
        };
        let enough_moves = Rule {
            related_input: PlayerInputType::Movement,
            rule_fn: Box::new(has_enough_moves),
        };

        let rules = vec![player_has_position, next_to_node, enough_moves];
        rules
    }
}

// ==================== RULES ====================

macro_rules! invalid_move {
    ($message: expr) => {
        Ok(ValidationResponse::new(false, Some($message.to_string())))
    };
}

macro_rules! valid_move {
    () => {
        Ok(ValidationResponse::new(true, None))
    };
}

fn has_enough_moves(
    game: &GameState,
    player_input: &PlayerInput,
) -> Result<ValidationResponse, String> {
    let player_result = game.get_player_with_unique_id(player_input.player_id);
    let player = match player_result {
        Ok(p) => p,
        Err(e) => return Err(e.to_string()),
    };

    if player.remaining_moves == 0 {
        return invalid_move!("The player has no remaining moves!");
    }

    let Some(position_node_id) = player.position_node_id else {
        return Err(format!("Player {} has no position!", player.unique_id));
    };

    let map = NodeMap::new();
    let current_node = match map.get_node_by_id(position_node_id) {
        Ok(node) => node,
        Err(e) => return Err(e),
    };

    let Some(related_node_id) = player_input.related_node_id else {
        return Err("There was no node to get cost to!".to_string());
    };

    let cost = match current_node.get_movement_cost_to_neighbour_with_id(related_node_id) {
        Ok(cost) => cost,
        Err(e) => return Err(e),
    };

    if player.remaining_moves < cost {
        return invalid_move!("The player has not enough remaining moves!");
    }

    valid_move!()
}

fn has_position(
    game: &GameState,
    player_input: &PlayerInput,
) -> Result<ValidationResponse, String> {
    match game.get_player_with_unique_id(player_input.player_id) {
        Ok(p) => {
            if p.position_node_id.is_none() {
                return invalid_move!("The player does not have a position!");
            }
            valid_move!()
        }
        Err(e) => Err(e.to_string()),
    }
}

fn next_node_is_neighbour(
    game: &GameState,
    player_input: &PlayerInput,
) -> Result<ValidationResponse, String> {
    match game.get_player_with_unique_id(player_input.player_id) {
        Ok(p) => {
            match p.position_node_id {
                Some(node_id) => {
                    let map = NodeMap::new();
                    match map.get_node_by_id(node_id) {
                        Ok(node) => {
                            let Some(related_node_id) = player_input.related_node_id else {
                                return Err("There was node to check if it's a neighbour!".to_string());
                            };
                            if !node.has_neighbour_with_id(related_node_id) {
                                return invalid_move!(format!(
                                    "The node {related_node_id} is not a neighbour of the player's position!",
                                ));
                            }
                        }
                        Err(e) => return Err(e),
                    };
                }
                None => return invalid_move!("The player does not have a position!"),
            }
            valid_move!()
        }
        Err(e) => Err(e.to_string()),
    }
}
