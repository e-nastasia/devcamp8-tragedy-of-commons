use game_move::GameMoveInput;
use hdk::prelude::*;

mod game_code;
#[allow(dead_code)]
mod game_move;
mod game_round;
mod game_session;
mod player_profile;
#[allow(dead_code)]
mod utils;

use crate::{
    game_session::GameSession,
    player_profile::{JoinGameInfo, PlayerProfile},
};

// This is part of Holochain data model definition, and here we specify
// what kinds of entries are available in our applicaton.
entry_defs![
    // Our implementation of game_code uses `anchor` helper method,
    // which requires us to add the Anchor and Path entry definitions
    Anchor::entry_def(),
    Path::entry_def(),
    // PlayerProfile Holochain entry definition callback. You wouldn't find a fn
    // named entry_def in player_profile.rs: this is one of the functions
    // generated by applying `#[hdk_entry]` macro to PlayerProfile struct
    player_profile::PlayerProfile::entry_def(),
    // GameSession Holochain entry definition callback
    game_session::GameSession::entry_def(),
    // GameRound Holochain entry definition callback
    game_round::GameRound::entry_def(),
    // GameMove Holochain entry definition callback
    game_move::GameMove::entry_def()
];

/// This is another macro applied to the function that follows, and we need it to
/// expose this function as part of our backend API
/// Note that this macro requires fn to accept input parameters, so if your fn
/// doesn't accept anything, write it's signature like this:
/// ```
/// #[hdk_extern]
/// fn foo(_: ()) -> ExternResult<EntryHash>
/// ```
/// This function is part of our publicly exposed API and it simply wraps
/// the corresponding function in game_code module.
#[hdk_extern]
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    game_code::create_game_code_anchor(short_unique_code)
}

/// Creates a user profile and links it to the game_code
#[hdk_extern]
pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHash> {
    player_profile::join_game_with_code(input)
}

/// Lists all players who are linked to the game_code
#[hdk_extern]
pub fn get_players_for_game_code(short_unique_code: String) -> ExternResult<Vec<PlayerProfile>> {
    player_profile::get_player_profiles_for_game_code(short_unique_code)
}

/// Creates a GameSession entry for the corresponding game_code
#[hdk_extern]
pub fn start_game_session_with_code(game_code: String) -> ExternResult<EntryHash> {
    game_session::start_game_session_with_code(game_code)
}

/// Lists all game sessions created by the agent who calls this fn
#[hdk_extern]
pub fn get_my_owned_sessions(_: ()) -> ExternResult<Vec<(EntryHash, GameSession)>> {
    game_session::get_my_own_sessions_via_source_query()
}

/// Creates a new move for the given round
#[hdk_extern]
pub fn make_new_move(input: GameMoveInput) -> ExternResult<HeaderHash> {
    game_move::new_move(input.resource_amount, input.round_hash)
}
