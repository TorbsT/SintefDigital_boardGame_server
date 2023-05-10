//! The game_core library contains all the data structures for the game and some of the game logic.
//!
//! The game_core library is the core of the game. It contains all the data structures for the game and some of the game logic.
//! The GameController struct in the game_controller module is the game manager and is what should be used to control all of the games on the server. It has all the neccessary functions to create and handle games.

/// The game_controller module contains the game controller struct and its methods related to controlling all the games of the server. And can be thought of as the server's game manager.
pub mod game_controller;
/// The game_data module contains all the data structures for the game and some of the game logic.
pub mod game_data;
/// The rule_checker module contains the trait for the rule checker.
pub mod rule_checker;
/// The situation_card_list module has the default situation cards for the game, including the objective/assignment cards for each situation card.
pub mod situation_card_list;
