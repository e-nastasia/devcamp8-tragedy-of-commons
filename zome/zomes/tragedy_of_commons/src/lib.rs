use game_move::GameMove;
use game_session::GameParams;
// use game_session::GameSession;
#[allow(unused_imports)]
use hdk::prelude::*;
#[allow(unused)]
use holo_hash::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use utils::{entry_from_element_create_or_update};

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
    Anchor::entry_def(),
    Path::entry_def(),
    game_session::GameSession::entry_def(),
    game_round::GameRound::entry_def(),
    game_move::GameMove::entry_def(),
    game_session::GameScores::entry_def(),
    PlayerProfile::entry_def()
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

#[hdk_entry(id = "player_profile", visibility = "public")]
#[derive(Clone)]
pub struct PlayerProfile {
    pub player_id: AgentPubKey,
    pub nickname: String,
}

#[hdk_extern]
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHashB64> {
    let anchor = anchor("GAME_CODES".into(), short_unique_code)?;
    Ok(EntryHashB64::from(anchor)) // or more Rust like: anchor.into())
}

#[hdk_extern]
pub fn join_game_with_code(input:JoinGameInfo) -> ExternResult<EntryHashB64> {
    info!("input: {:?}", input);
    info!("game code: {:?}", input.gamecode);
    let anchor = anchor("GAME_CODES".into(), input.gamecode)?;
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
    create_link(anchor.clone().into(), player_profile_entry_hash.into(), ())?;
    debug!("link created");
    Ok(EntryHashB64::from(anchor)) // or more Rust like: anchor.into())
}

#[hdk_extern]
pub fn get_players_for_game_code(short_unique_code: String) -> ExternResult<Vec<PlayerProfile>> {
    // Ok(vec!["Anipur".into(), "Bob".into()]);

    debug!("get profiles");
    let player_profiles = get_player_profiles_for_game_code(short_unique_code)?;

    // debug!("filter profiles to extract nickname");
    let players:Vec<String> = player_profiles.iter().map(|x| x.nickname.clone()).collect();
    debug!("players: {:?}", players);
    debug!("profiles {:?}", player_profiles);
    Ok(player_profiles) // or more Rust like: anchor.into())
}

#[hdk_extern]
pub fn get_game_session_info(game_session_header_hash: HeaderHash)-> ExternResult<GameSessionInfo>{
    //get game session
    //get game round
    let x = GameSessionInfo{
        round_num:1,
        current_round_hash:(),
        entry_hash:(),
        current_round_state:"IN_PROGESS".into(),  //FINISHED
        resources_left:100,
    };

    Ok(x)
}

pub fn get_player_profiles_for_game_code(
    short_unique_code: String,
) -> ExternResult<Vec<PlayerProfile>> {

    let anchor = anchor("GAME_CODES".into(), short_unique_code)?;
    debug!("anchor: {:?}", anchor);
    let links: Links = get_links(anchor, None)?;
    debug!("links: {:?}", links);
    let mut players = vec![];
    for link in links.into_inner() {
        debug!("link: {:?}", link);
        let element: Element = get(link.target, GetOptions::default())?
            .ok_or(WasmError::Guest(String::from("Entry not found")))?;
        let entry_option = element.entry().to_app_option()?;
        let entry: PlayerProfile = entry_option.ok_or(WasmError::Guest(
            "The targeted entry is not agent pubkey".into(),
        ))?;
        players.push(entry);
    }

    Ok(players) // or more Rust like: anchor.into())
}

/// Placeholder function that can be called from UI/test, until invitation zoom is added.
#[hdk_extern]
pub fn start_game_session_with_code(game_code: String) -> ExternResult<HeaderHashB64> {
    let anchor = anchor("GAME_CODES".into(), game_code.clone())?;
    debug!("anchor: {:?}",anchor);
    let players = get_player_profiles_for_game_code(game_code)?;
    debug!("players: {:?}", players);
    start_default_session(players, anchor)
}

#[hdk_extern]
pub fn game_session_with_code_exists(game_code: String) -> ExternResult<String> {
    let anchor = anchor("GAME_CODES".into(), game_code.clone())?;
    let linktag = LinkTag::new("GAME_SESSION");
    let links: Links = get_links(anchor, Some(linktag))?;
    debug!("links: {:?}", links);
    if links.clone().into_inner().len() > 1 {  // TODO find alternative for clone to get len
        return Err(WasmError::Guest(String::from("More than one link from anchor to game session. Should not happen.")))
    }
    let mut game_session_option:Option<GameSession> = None;
    for link in links.into_inner() {
        debug!("link: {:?}", link);
        let element: Element = get(link.target, GetOptions::latest())?
            .ok_or(WasmError::Guest(String::from("Entry not found")))?;
        game_session_option = element.entry().to_app_option()?;

    }

    let gs: GameSession = match game_session_option {
        Some(x) => x,
        None => return Ok("STILL WAITING".into()),
    };

    match gs.status {
        SessionState::InProgress => Ok("IN_PROGRESS".into()),
        SessionState::Finished => Ok("FINISHED".into()),
    }
}



pub fn start_default_session(player_list: Vec<PlayerProfile>, anchor:EntryHash) -> ExternResult<HeaderHashB64> {
    let game_params = GameParams {
        regeneration_factor: 1.1,
        start_amount: 100,
        num_rounds: 3,
        resource_coef: 3,
        reputation_coef: 2,
    };
    let players: Vec<AgentPubKey> = player_list.iter().map(|x| x.player_id.clone()).collect(); //convert_keys_from_b64(&player_list);
    debug!("player agentpubkeys: {:?}", players);
    let round_zero = game_session::new_session(players, game_params, anchor);
    debug!("new session created: {:?}", round_zero);
    match round_zero {
        Ok(hash) => Ok(HeaderHashB64::from(hash)),
        Err(error) => Err(error),
    }
}

/// Function to call when player wants to start a new game and has already selected
/// invitees for this game. This function is only supposed to handle invite zome integration
/// and it shouldn't be really creating a new GameSession entry.
// #[hdk_extern]
// pub fn propose_new_session() -> ExternResult<HeaderHash> {}

// /// Function to call by the invite zome once all invites are taken care of
// /// and we can actually create the GameSession and start playing
// pub fn create_new_session(input: GameSessionInput) -> ExternResult<HeaderHashB64> {
//     let players: Vec<AgentPubKey> = convert_keys_from_b64(&input.players);
//     let game_params = input.game_params;
//     convert(game_session::new_session(players, game_params))
// }

// TODO: think of better naming to distinguish between sessions "as owner" and "as player"
/// Function to list all game sessions that the caller has created
#[hdk_extern]
pub fn get_my_owned_sessions(_: ()) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    //game_session::get_sessions_with_tags(vec![OWNER_SESSION_TAG])
    game_session::get_my_own_sessions_via_source_query()
}


/// Function to make a new move in the game specified by input
#[hdk_extern]
pub fn make_new_move(input: GameMoveInput) -> ExternResult<HeaderHashB64> {
    //TODO convert
    convert(game_move::new_move(
        input.resource_amount,
        input.previous_round.into(),
    ))
}

/// Function to call from the UI on a regular basis to try and close the currently
/// active GameRound. It will check the currently available GameRound state and then
/// will close it if it's possible. If not, it will return None
#[hdk_extern]
pub fn try_to_close_round(prev_round_hash: HeaderHashB64) -> ExternResult<HeaderHashB64> {
    // TODO: this should probably go to the game_round.rs instead
    convert(game_round::try_to_close_round(prev_round_hash.into()))
}

fn convert(result: ExternResult<HeaderHash>) -> ExternResult<HeaderHashB64> {
    match result {
        Ok(hash) => return Ok(HeaderHashB64::from(hash)),
        Err(error) => return Err(error),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SignalTest {
    pub content: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct JoinGameInfo {
    pub gamecode: String,
    pub nickname: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameSessionInfo {
    pub entry_hash: HeaderHash,
    pub round_num: i32,
    pub resources_left: i32,
    pub current_round_hash: HeaderHash,
    pub current_round_state: String,
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
    // debug!("validating game move");
    let x: GameMove = entry_from_element_create_or_update(&validate_data.element)?;
    // trace!("resources {}", x.resources);
    if x.resources < 0 {
        return Ok(ValidateCallbackResult::Invalid(
            "You cannot insert resources back".into(),
        ));
    }
    Ok(ValidateCallbackResult::Valid)
}
