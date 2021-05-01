use hdk::prelude::*;
//use holo_hash::EntryHashB64;
mod game_move;
mod game_round;
#[allow(unused_imports)]
#[allow(dead_code)]
mod game_session;
mod types;
mod utils;

// TODO: Actually code the zome, all this code is just for reference and quick copy-paste

pub fn err(reason: &str) -> WasmError {
    WasmError::Guest(String::from(reason))
}

entry_defs![
    Path::entry_def(),
    game_session::GameSession::entry_def(),
    game_round::GameRound::entry_def(),
    game_move::GameMove::entry_def()
];
