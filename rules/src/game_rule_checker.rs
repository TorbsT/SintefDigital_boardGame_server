use std::ops::ControlFlow;

use game_core::{
    game_data::{GameState, PlayerInput, PlayerInputType},
    rule_checker::RuleChecker,
};

type RuleFn = Box<dyn Fn(&GameState, &PlayerInput) -> Result<bool, String> + Send + Sync>;

struct Rule {
    pub related_input: PlayerInputType,
    pub rule_fn: RuleFn,
}

pub struct GameRuleChecker {
    rules: Vec<Rule>,
}

impl RuleChecker for GameRuleChecker {
    fn is_input_valid(&self, game: &GameState, player_input: &PlayerInput) -> Result<bool, String> {
        let mut error_str = String::new();
        let foreach_status = &self.rules.iter().try_for_each(|rule| {
            if rule.related_input != player_input.input_type {
                return ControlFlow::Continue(());
            }

            match (rule.rule_fn)(game, player_input) {
                Ok(is_valid) => {
                    if !is_valid {
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
            if error_str == String::new() {
                return Ok(false);
            }
            return Err(error_str);
        }
        Ok(true)
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
        let enough_moves = Rule {
            related_input: PlayerInputType::Movement,
            rule_fn: Box::new(has_enough_moves),
        };

        let rules = vec![enough_moves];
        rules
    }
}

fn has_enough_moves(game: &GameState, player_input: &PlayerInput) -> Result<bool, String> {
    let player_result = game.get_player_with_unique_id(player_input.player_id);
    let player = match player_result {
        Ok(p) => p,
        Err(e) => return Err(e.to_string()),
    };

    if player.remaining_moves == 0 {
        return Ok(false);
    }
    todo!("Check if player - movement cost >= 0");
}

// ============= Helper functions ===============
