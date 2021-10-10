#[allow(unused_imports)]
use hdk::prelude::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[allow(unused_imports)]
use crate::{
    game_move::{GameMove, GameMoveInput},
    game_round::GameRoundInfo,
    game_session::{
        GameParams, GameSession, GameSessionInput, GameSignal, SessionState, SignalPayload,
        OWNER_SESSION_TAG,
    },
    player_profile::{JoinGameInfo, PlayerProfile},
    utils::{convert, entry_from_element_create_or_update},
};
mod error;
mod game_code;
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
mod player_profile;
mod types;
mod utils;

pub fn err(reason: &str) -> WasmError {
    WasmError::Guest(String::from(reason))
}

entry_defs![
    Anchor::entry_def(),
    Path::entry_def(),
    game_session::GameSession::entry_def(),
    game_round::GameRound::entry_def(),
    game_move::GameMove::entry_def(),
    game_session::GameScores::entry_def(),
    player_profile::PlayerProfile::entry_def()
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
        .with_target(false)
        .with_max_level(Level::TRACE)
        .without_time()
        .compact()
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
    player_profile::get_players_for_game_code(short_unique_code)
}

#[hdk_extern]
pub fn start_game_session_with_code(game_code: String) -> ExternResult<EntryHash> {
    game_session::start_game_session_with_code(game_code)
}

// #[hdk_extern]
// pub fn current_round_info(game_round_entry_hash: EntryHash) -> ExternResult<GameRoundInfo> {
//     game_round::current_round_info(game_round_entry_hash)
// }

#[hdk_extern]
pub fn current_round_for_game_code(game_code: String) -> ExternResult<Option<EntryHash>> {
    game_round::current_round_for_game_code(game_code)
}

/// Function to call when player wants to start a new game and has already selected
/// invitees for this game. This function is only supposed to handle invite zome integration
/// and it shouldn't be really creating a new GameSession entry.
// #[hdk_extern]
// pub fn propose_new_session() -> ExternResult<HeaderHash> {}

// /// Function to call by the invite zome once all invites are taken care of
// /// and we can actually create the GameSession and start playing
// pub fn create_new_session(input: GameSessionInput) -> ExternResult<HeaderHash> {
//     let players: Vec<AgentPubKey> = convert_keys_from_b64(&input.players);
//     let game_params = input.game_params;
//     convert(game_session::new_session(players, game_params))
// }

/// Function to list all game sessions that the caller has created
#[hdk_extern]
pub fn get_my_owned_sessions(_: ()) -> ExternResult<Vec<(EntryHash, GameSession)>> {
    //game_session::get_sessions_with_tags(vec![OWNER_SESSION_TAG])
    game_session::get_my_own_sessions_via_source_query()
}

/// Function to make a new move in the game specified by input
#[hdk_extern]
pub fn make_new_move(input: GameMoveInput) -> ExternResult<HeaderHash> {
    game_move::new_move(input.resource_amount, input.previous_round)
}

/// Function to call from the UI on a regular basis to try and close the currently
/// active GameRound. It will check the currently available GameRound state and then
/// will close it if it's possible. If not, it will return None
#[hdk_extern]
pub fn try_to_close_round(prev_round_hash: EntryHash) -> ExternResult<GameRoundInfo> {
    // TODO: this should probably go to the game_round.rs instead
    game_round::try_to_close_round(prev_round_hash.into())
}

#[hdk_extern]
pub fn validate(_validation_data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    // Ok(ValidateCallbackResult::Invalid("computer says no")
    // Ok(ValidateCallbackResult::UnresolvedDependencies("something is missing"))
    // debug!("validate general");
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate_create(_validation_data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    // all creates are valid
    // debug!("validate create");
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate_create_entry_game_move(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_move::validate_create_entry_game_move(validate_data)
}

#[hdk_extern]
pub fn validate_update_entry_game_move(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_move::validate_update_entry_game_move(validate_data)
}

#[hdk_extern]
pub fn validate_delete_entry_game_move(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_move::validate_delete_entry_game_move(validate_data)
}

#[hdk_extern]
pub fn validate_update_entry_game_round(
    data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_round::validate_update_entry_game_round(data)
}
