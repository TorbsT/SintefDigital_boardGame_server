//! Contains most the structs used in the game.

/// The cost_tuple module contains the CostTuple struct which describes the Traffic in a District.
pub mod cost_tuple;
/// The district_modifier module contains the DistrictModifier struct which describes a DistrictModifier.
pub mod district_modifier;
/// The edge_restriction module contains the EdgeRestriction struct which describes an EdgeRestriction.
pub mod edge_restriction;
/// The game_state module contains the GameState struct which describes the state of the game.
pub mod gamestate;
/// The neighbour_relationship module contains the NeighbourRelationship struct which describes the relationship between two nodes.
pub mod neighbour_relationship;
/// The new_game_info module contains the NewGameInfo struct which describes the information needed to create a new game.
pub mod new_game_info;
/// The node_map module contains the NodeMap struct which describes the map of the game.
pub mod node_map;
/// The node module contains the Node struct which describes a node.
pub mod node;
/// The player_input module contains the PlayerInput struct which describes the input of a player.
pub mod player_input;
/// The player_objective_card module contains the PlayerObjectiveCard struct which describes a player objective card.
pub mod player_objective_card;
/// The player module contains the Player struct which describes a player.
pub mod player;
/// The situation_card_list module contains the SituationCardList struct which describes a list of situation cards.
pub mod situation_card_list;
/// The situation_card module contains the SituationCard struct which describes a situation card for the game, it also includes [`PlayerObjectiveCard`].
/// 
/// [`PlayerObjectiveCard`]: struct.PlayerObjectiveCard.html
pub mod situation_card;