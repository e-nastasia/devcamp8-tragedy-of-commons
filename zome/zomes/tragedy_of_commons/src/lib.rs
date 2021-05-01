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

// give unrestricted access to recv_remote_signal, which is needed for sending remote signals
#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;

    Ok(InitCallbackResult::Pass)
}

// function required to process remote signals see hdk/src/p2p.rs
#[hdk_extern]
fn recv_remote_signal(signal: SerializedBytes) -> ExternResult<()> {
    emit_signal(&signal)?;
    Ok(())
}