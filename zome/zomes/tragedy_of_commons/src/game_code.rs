use hdk::prelude::holo_hash::{EntryHashB64, HeaderHashB64};
use hdk::prelude::*;

pub const GAME_CODES_ANCHOR: &str = "GAME_CODES";

pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHashB64> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), short_unique_code)?;
    Ok(EntryHashB64::from(anchor)) // or more Rust like: anchor.into())
}

/// Creates GameSession with the game_code and game_params
// TODO(e-nastasia): actually add game_params to be used for creation
pub fn start_game_session_with_code(game_code: String) -> ExternResult<HeaderHashB64> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), game_code.clone())?;
    debug!("anchor: {:?}", anchor);
    let players = crate::player_profile::get_player_profiles_for_game_code(game_code)?;
    debug!("players: {:?}", players);
    crate::game_session::start_default_session(players, anchor)
}
