use serde::{Deserialize, Serialize};
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
    ModifyEdgeRestrictions,
    SetPlayerBusBool,
}