use game_session::GameParams;
// use game_session::GameSession;
use hdk::prelude::*;
#[allow(unused)]
use holo_hash::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use utils::convert_keys_from_b64;

#[allow(unused_imports)]
use crate::{
    game_move::GameMoveInput,
    game_session::{
        GameSession, GameSessionInput, GameSignal, SessionState, SignalPayload, OWNER_SESSION_TAG,
        PARTICIPANT_SESSION_TAG,
    },
};
mod error;
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(unused)]
mod game_move;
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(unused)]
mod game_round;
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(unused)]
mod game_session;
mod types;
mod utils;

pub fn err(reason: &str) -> WasmError {
    WasmError::Guest(String::from(reason))
}

entry_defs![
    Path::entry_def(),
    game_session::GameSession::entry_def(),
    game_round::GameRound::entry_def(),
    game_move::GameMove::entry_def(),
    game_session::GameScores::entry_def()
];

// give unrestricted access to recv_remote_signal, which is needed for sending remote signals
#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));

    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        access: ().into(), // empty access converts to unrestricted
        functions,
    })?;

    // i have no idea where to put the tracing config, as all examples suggest main
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Ok(InitCallbackResult::Pass)
}

// function required to process remote signals see hdk/src/p2p.rs
#[hdk_extern]
fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
    debug!("Received remote signal {:?}", signal);
    let game_signal_result: Result<GameSignal, SerializedBytesError> = signal.decode();
    //debug!("Received REMOTE signal {:?}", sig);
    match game_signal_result {
        Ok(a) => emit_signal(a),
        Err(_) => Err(WasmError::Guest("Remote signal failed".into())),
    }
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone)]
pub struct ZomeInput {
    number: i32,
}

#[hdk_extern]
pub fn start_test_fn(_z: ZomeInput) -> ExternResult<i32> {
    Ok(42)
}

/// Placeholder function that can be called from UI/test, until invitation zoom is added.
#[hdk_extern]
pub fn start_dummy_session(player_list: Vec<AgentPubKeyB64>) -> ExternResult<HeaderHash> {
    let game_params = GameParams {
        regeneration_factor: 1.1,
        start_amount: 100,
        num_rounds: 3,
        resource_coef: 3,
        reputation_coef: 2,
    };
    let players = convert_keys_from_b64(&player_list);
    game_session::new_session(players, game_params)
}

/// Function to call when player wants to start a new game and has already selected
/// invitees for this game. This function is only supposed to handle invite zome integration
/// and it shouldn't be really creating a new GameSession entry.
// #[hdk_extern]
// pub fn propose_new_session() -> ExternResult<HeaderHash> {}

/// Function to call by the invite zome once all invites are taken care of
/// and we can actually create the GameSession and start playing
pub fn create_new_session(input: GameSessionInput) -> ExternResult<HeaderHash> {
    let players: Vec<AgentPubKey> = convert_keys_from_b64(&input.players);
    let game_params = input.game_params;
    game_session::new_session(players, game_params)
}

// TODO: think of better naming to distinguish between sessions "as owner" and "as player"
/// Function to list all game sessions that the caller has created
#[hdk_extern]
pub fn get_my_owned_sessions(_: ()) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    game_session::get_sessions_with_tags(vec![OWNER_SESSION_TAG])
}

/// Function to list game sessions in which caller has been a participant.
#[hdk_extern]
pub fn get_my_played_sessions(_: ()) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    game_session::get_sessions_with_tags(vec![PARTICIPANT_SESSION_TAG])
}

/// Function to list all game sessions in which caller was involved, both as
/// an owner and as a participant
#[hdk_extern]
pub fn get_all_my_sessions(_: ()) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    game_session::get_sessions_with_tags(vec![OWNER_SESSION_TAG, PARTICIPANT_SESSION_TAG])
}

/// Function to list all active sessions where caller is either owner or participant
#[hdk_extern]
pub fn get_my_active_sessions(_: ()) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    game_session::get_sessions_with_status(SessionState::InProgress)
}

/// Function to make a new move in the game specified by input
#[hdk_extern]
pub fn make_new_move(input: GameMoveInput) -> ExternResult<HeaderHash> {
    game_move::new_move(input.resource_amount, input.previous_round)
}

/// Function to call from the UI on a regular basis to try and close the currently
/// active GameRound. It will check the currently available GameRound state and then
/// will close it if it's possible. If not, it will return None
pub fn try_to_close_round(prev_round_hash: HeaderHashB64) -> ExternResult<HeaderHashB64> {
    // TODO: this should probably go to the game_round.rs instead
    let x = game_round::try_to_close_round(prev_round_hash.into());
    match x {
        Ok(hash) => Ok(HeaderHashB64::from(hash)),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SignalTest {
    pub content: String,
    pub value: String,
}
