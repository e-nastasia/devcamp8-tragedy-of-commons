use hdk::prelude::*;
use hdk::prelude::holo_hash::{EntryHashB64, HeaderHashB64};

use crate::player_profile::PlayerProfile;

pub const GAME_CODES_ANCHOR: &str = "GAME_CODES";

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct JoinGameInfo {
    pub gamecode: String,
    pub nickname: String,
}

pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHashB64> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), short_unique_code)?;
    Ok(EntryHashB64::from(anchor)) // or more Rust like: anchor.into())
}

pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHashB64> {
    info!("input: {:?}", input);
    info!("game code: {:?}", input.gamecode);
    let anchor = anchor(GAME_CODES_ANCHOR.into(), input.gamecode)?;
    debug!("anchor created {:?}", &anchor);
    let agent = agent_info()?;
    debug!("agent {:?}", agent.clone());
    let player_profile = PlayerProfile {
        player_id: agent.agent_initial_pubkey, // bad design for real apps 1/ initial_pubkey is linked to app itself, so no roaming profile 2/ lost if app is reinstalled (= basicly new user)
        nickname: input.nickname,
    };
    create_entry(&player_profile)?;
    debug!("profile created");
    let player_profile_entry_hash = hash_entry(&player_profile)?;
    debug!("profile entry hash {:?}", &player_profile_entry_hash);
    create_link(
        anchor.clone().into(),
        player_profile_entry_hash.into(),
        LinkTag::new("PLAYER"),
    )?;
    debug!("link created");
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
