use hdk::prelude::*;

// We apply macro to the the line of code that follows, allowing to compile code
// from the game_code module even if some of this code is unused.
// (Default compiler's behavior is to treat unused code as errors)
// We'll remove this line later once we start using all code from the game_code module.
// "unused" here is name of the lint group, and there are actually a lot of those!
// Check out this link for more details:
// https://doc.rust-lang.org/rustc/lints/groups.html
#[allow(unused)]
mod game_code;
#[allow(unused)]
mod player_profile;

use crate::player_profile::{JoinGameInfo, PlayerProfile};

entry_defs![
    player_profile::PlayerProfile::entry_def()
];

// This is another macro applied to the function that follows, and we need it to
// expose this function as part of our backend API
#[hdk_extern]
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    game_code::create_game_code_anchor(short_unique_code)
}

#[hdk_extern]
pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHash> {
    player_profile::join_game_with_code(input)
}

#[hdk_extern]
pub fn get_players_for_game_code(short_unique_code: String) -> ExternResult<Vec<PlayerProfile>> {
    player_profile::get_player_profiles_for_game_code(short_unique_code)
}
