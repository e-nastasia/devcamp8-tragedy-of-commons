use crate::error::Error;
use crate::types::{new_player_stats, PlayerStats, ResourceAmount};
use crate::utils::{entry_from_element_create_or_update, entry_hash_from_element};
use crate::PlayerProfile;
use crate::{
    game_code::get_game_code_anchor,
    game_round::{GameRound, RoundState},
    utils::convert_keys_from_b64,
};

use hdk::prelude::*;
use std::{collections::HashMap, time::SystemTime};

pub const OWNER_SESSION_TAG: &str = "MY_GAMES";
pub const GAME_CODE_TO_SESSION_TAG: &str = "GAME_SESSION";
pub const SESSION_TO_ROUND_TAG: &str = "GAME_ROUND";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    InProgress,
    Lost { last_round: EntryHash },
    // TODO: when validating things, check that last game round is finished to verify
    // that session itself is finished
    Finished { last_round: EntryHash },
}

#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct GameParams {
    pub regeneration_factor: f32,
    pub start_amount: ResourceAmount,
    pub num_rounds: u32,
    pub resource_coef: u32,
    pub reputation_coef: u32,
}

#[hdk_entry(id = "game_session", visibility = "public")]
#[derive(Clone)]
pub struct GameSession {
    pub owner: AgentPubKey, // who started the game
    // pub created_at: Timestamp,     // when the game was started
    pub status: SessionState,      // how the game is going
    pub game_params: GameParams,   // what specific game are we playing
    pub players: Vec<AgentPubKey>, // who is playing
    pub scores: PlayerStats,       // end scores
    pub anchor: EntryHash,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameSessionInput {
    pub game_params: GameParams,
    pub players: Vec<AgentPubKey>,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SignalPayload {
    pub game_session_entry_hash: EntryHash,
    pub round_entry_hash_update: EntryHash,
}

#[hdk_entry(id = "game_scores", visibility = "public")]
#[derive(Clone)]
pub struct GameScores {
    pub game_session: GameSession,
    pub game_session_entry_hash: EntryHash,
    //TODO add the actual results :-)
}

/*
validation rules:

- make sure session is created only when invites are answered and there's at least one accepted
    - TODO: add addresses of accepted invites into game session, later

*/

impl GameSession {
    // called in different contexts:
    // if validation: if round isn't available, validation sin't finished
    // if session state update: round is available
    pub fn update_state(&self, _game_round: GameRound) {
        // this is called every time after GameRound is created

        // if round is lost <= 0:
        //  game session is lost
        // elif number round == num_rounds:
        //  game session is finished
        // else:
        //  game session is in progress
    }
}

/// Creates GameSession with the game_code and game_params
// TODO(e-nastasia): actually add game_params to be used for creation
pub fn start_game_session_with_code(game_code: String) -> ExternResult<EntryHash> {
    let anchor = get_game_code_anchor(game_code.clone())?;
    debug!("anchor: {:?}", anchor);
    let players = crate::player_profile::get_player_profiles_for_game_code(game_code)?;
    debug!("players: {:?}", players);
    start_default_session(players, anchor)
}

// TODO(e-nastasia) This is a placeholder fn that can be refactored once
// the UI is providing game params. Or we can leave it to separate retrieval
// of the players from the actual session create. Anyway, GameParams have to go!
fn start_default_session(
    player_list: Vec<PlayerProfile>,
    anchor: EntryHash,
) -> ExternResult<EntryHash> {
    let game_params = GameParams {
        regeneration_factor: 1.1,
        start_amount: 100,
        num_rounds: 3,
        resource_coef: 3,
        reputation_coef: 2,
    };
    let players: Vec<AgentPubKey> = player_list.iter().map(|x| x.player_id.clone()).collect(); //convert_keys_from_b64(&player_list);
    debug!("player agentpubkeys: {:?}", players);
    let round_zero = new_session(players, game_params, anchor);
    debug!("new session created: {:?}", round_zero);
    match round_zero {
        Ok(hash) => Ok(hash),
        Err(error) => Err(error),
    }
}

/// Create a new GameSession with the confirmed players (who accepted their invites).
/// NOTE: we're only creating session for those who accepted and only if there are at
/// least two of them -- otherwise there won't be any turns.
pub fn new_session(
    players: Vec<AgentPubKey>,
    game_params: GameParams,
    anchor: EntryHash,
) -> ExternResult<EntryHash> {
    // TODO: get timestamp as systime

    info!("creating new game session");
    // agent that starts new game
    let agent_info_owner = agent_info()?;
    // create entry for game session
    let game_session = GameSession {
        owner: agent_info_owner.agent_initial_pubkey.clone(),
        status: SessionState::InProgress,
        game_params: game_params,
        players: players.clone(),
        scores: PlayerStats::new(),
        anchor: anchor.clone(),
    };
    let game_session_header_hash = create_entry(&game_session)?;
    let game_session_entry_hash = hash_entry(&game_session)?;

    info!("linking owner to game session");
    debug!(
        "================= Creating link from OWNER address {:?} to game session {:?}",
        agent_info_owner.agent_initial_pubkey.clone(),
        game_session_entry_hash.clone()
    );
    // create link from session owner's address to the game session entry
    create_link(
        agent_info_owner.agent_initial_pubkey.clone().into(),
        game_session_entry_hash.clone(),
        LinkTag::new(OWNER_SESSION_TAG),
    )?;

    info!("linking game code anchor to game session");
    // create link from session owner's address to the game session entry
    create_link(
        anchor.into(),
        game_session_entry_hash.clone(),
        LinkTag::new(GAME_CODE_TO_SESSION_TAG),
    )?;

    // create game round results for round 0
    // this is starting point for all the game moves of round 1 to reference (implicit link)
    let no_moves: Vec<EntryHash> = vec![];

    // TODO: create a link from session to game round entry to make the round discoverable
    let round_zero = GameRound::new(
        0,
        game_session_entry_hash.clone(),
        game_session.game_params.start_amount,
        0,
        0,
            // new_player_stats(&players),
            // no_moves,
    );
    let header_hash_round_zero = create_entry(&round_zero)?;
    let entry_hash_round_zero = hash_entry(&round_zero)?;

    create_link(
        game_session_entry_hash.clone(),
        entry_hash_round_zero.clone(),
        LinkTag::new(SESSION_TO_ROUND_TAG),
    );

    // use remote signals from RSM to send a real-time notif to invited players
    //  ! using remote signal to ping other holochain backends, instead of emit_signal
    //  that would talk with the UI
    // NOTE: we're sending signals to notify that a new round has started and
    // that players need to make their moves
    // WARNING: remote_signal is fire and forget, no error if it fails, might be a weak point if this were production happ
    let signal_payload = SignalPayload {
        game_session_entry_hash: game_session_entry_hash.into(),
        round_entry_hash_update: entry_hash_round_zero.clone().into(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    // Since we're storing agent keys as AgentPubKey, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, players.clone())?;
    debug!("sending signal to {:?}", players);

    Ok(entry_hash_round_zero)
}

pub fn end_game(
    game_session: &GameSession,
    game_session_header_hash: &HeaderHash,
    last_round: &GameRound,
    last_round_entry_hash: &EntryHash,
    round_state: &RoundState,
) -> ExternResult<EntryHash> {
    info!("ending game");
    // last_round contains end results
    // so no creates or update are necessary
    // only a signal to all players that game has ended
    // players that miss the signal should have their UI poll GameRound
    // based on that content it can be derive if the game has ended or not

    info!("updating game session: setting finished state and adding player stats");
    let game_status = if round_state.resources_taken_round <= 0 {
        SessionState::Lost {
            last_round: last_round_entry_hash.clone(),
        }
    } else {
        SessionState::Finished {
            last_round: last_round_entry_hash.clone(),
        }
    };
    //update chain for game session entry
    let game_session_update = GameSession {
        owner: game_session.owner.clone(),
        status: game_status,
        game_params: game_session.game_params.clone(),
        players: game_session.players.clone(),
        scores: round_state.player_stats.clone(),
        anchor: game_session.anchor.clone(),
    };
    let game_session_header_hash_update =
        update_entry(game_session_header_hash.clone(), &game_session_update)?;
    let game_session_entry_hash_update = hash_entry(&game_session_update)?;
    debug!(
        "updated game session header hash: {:?}",
        game_session_header_hash_update.clone()
    );
    debug!(
        "updated game session entry hash: {:?}",
        game_session_entry_hash_update.clone()
    );

    info!("signaling player game has ended");
    let signal_payload = SignalPayload {
        game_session_entry_hash: game_session_entry_hash_update.clone(),
        round_entry_hash_update: last_round_entry_hash.clone().into(),
    };
    let signal = ExternIO::encode(GameSignal::GameOver(signal_payload))?;
    // Since we're storing agent keys as AgentPubKey, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:?}", game_session.players.clone());

    Ok(game_session_entry_hash_update.clone())
}

pub fn get_sessions_with_tags(link_tags: Vec<&str>) -> ExternResult<Vec<(EntryHash, GameSession)>> {
    let agent_key: EntryHash = agent_info()?.agent_latest_pubkey.into();
    let mut results_tmp: Vec<Link> = vec![];
    for lt in link_tags {
        let mut links = get_links(agent_key.clone(), Some(LinkTag::new(lt)))?.into_inner();
        results_tmp.append(&mut links);
    }

    let results = results_tmp
        .iter()
        .map(|link| {
            let result = get_game_session(link.target.clone())?;
            Ok((link.target.clone(), result))
        })
        .collect::<ExternResult<Vec<(EntryHash, GameSession)>>>()?;

    Ok(results)
}

pub fn get_my_own_sessions_via_source_query() -> ExternResult<Vec<(EntryHash, GameSession)>> {
    let filter = ChainQueryFilter::new()
        .include_entries(true)
        .entry_type(EntryType::App(AppEntryType::new(
            entry_def_index!(GameSession)?,
            zome_info()?.zome_id,
            EntryVisibility::Public,
        )));

    let list_of_elements = query(filter)?;
    let mut list_of_tuples: Vec<(EntryHash, GameSession)> = vec![];
    for el in list_of_elements {
        let gs: GameSession = entry_from_element_create_or_update(&el)?;
        let gs_entry_hash: EntryHash = entry_hash_from_element(&el)?.to_owned();
        list_of_tuples.push((gs_entry_hash, gs));
    }
    Ok(list_of_tuples)
}

pub fn get_sessions_with_status(
    target_state: SessionState,
) -> ExternResult<Vec<(EntryHash, GameSession)>> {
    let all_sessions = get_sessions_with_tags(vec![OWNER_SESSION_TAG])?;

    let results = all_sessions
        .into_iter()
        .filter(|entry| entry.1.status == target_state)
        .collect::<Vec<(EntryHash, GameSession)>>();

    Ok(results)
}

fn get_game_session(game_result_hash: EntryHash) -> ExternResult<GameSession> {
    let element = get(game_result_hash.clone(), GetOptions::default())?.ok_or(WasmError::Guest(
        format!("Could not get game session at: {}", game_result_hash).into(),
    ))?;

    let game_result: GameSession = element
        .entry()
        .to_app_option()?
        .ok_or(WasmError::Guest("Could not get game result".into()))?;

    Ok(game_result)
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
#[serde(tag = "signal_name", content = "signal_payload")]
pub enum GameSignal {
    StartNextRound(SignalPayload),
    GameOver(SignalPayload),
}
