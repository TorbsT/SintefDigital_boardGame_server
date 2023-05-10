use serde::{Deserialize, Serialize};

use super::player::Player;

#[derive(Clone, Serialize, Deserialize)]
pub struct NewGameInfo {
    pub host: Player,
    pub name: String,
}

