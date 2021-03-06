use crate::game_code::calculate_game_code_anchor_entry_hash;
use crate::game_move::{finalize_moves, get_moves_for_round, GameMove};
use crate::game_session::{
    GameParams, GameScores, GameSession, SessionState, GameSignal, SignalPayload,
};
use crate::types::{player_stats_from_moves, PlayerStats, ResourceAmount};
use crate::utils::{
    check_agent_is_player_current_session, convert_keys_from_b64,
    entry_from_element_create_or_update, entry_hash_from_element, must_get_entry_struct,
    must_get_header_and_entry,
};
use hdk::prelude::*;
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundState {
    pub resources_left: ResourceAmount,
    pub resources_taken: ResourceAmount,
    pub resources_grown: ResourceAmount,
    pub player_stats: PlayerStats,
}

#[hdk_entry(id = "game_round", visibility = "public")]
#[derive(Clone, PartialEq, Eq)]
pub struct GameRound {
    pub round_num: u32,
    pub session: EntryHash,
    pub state: RoundState,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameRoundInfo {
    pub round_num: u32,
    pub resources_left: Option<ResourceAmount>,
    pub resources_taken_round: Option<ResourceAmount>,
    pub resources_grown_round: Option<ResourceAmount>,
    pub current_round_entry_hash: Option<EntryHash>,
    pub prev_round_entry_hash: Option<EntryHash>,
    pub game_session_hash: Option<EntryHash>,
    pub next_action: String,
    pub moves: Vec<(ResourceAmount, String, AgentPubKey)>,
}

impl RoundState {
    /// Creates a new RoundState instance with the provided input
    pub fn new(
        resources_left: ResourceAmount,
        resources_taken: ResourceAmount,
        resources_grown: ResourceAmount,
        player_stats: PlayerStats,
    ) -> RoundState {
        RoundState {
            resources_left,
            resources_taken,
            resources_grown,
            player_stats,
        }
    }
}

impl GameRound {
    /// Creates a new GameRound instance with the provided input
    pub fn new(
        round_num: u32,
        session: EntryHash,
        resources_left: ResourceAmount,
        resources_taken: ResourceAmount,
        resources_grown: ResourceAmount,
        // previous_round_moves: Vec<EntryHash>,
    ) -> GameRound {
        let state = RoundState::new(
            resources_left,
            resources_taken,
            resources_grown,
            PlayerStats::new(),
        );
        GameRound {
            round_num,
            session,
            state,
        }
    }
}

// NOTE: this fn would be used both in validation and when creating game round entries
// so it has to be very lightweight and can not make any DHT queries
pub fn calculate_round_state(
    last_round: &GameRound,
    params: &GameParams,
    player_moves: Vec<GameMove>,
) -> RoundState {
    // resources
    let consumed_resources_in_round: ResourceAmount =
        player_moves.iter().map(|x| x.resources).sum();
    let resources_left = last_round.state.resources_left - consumed_resources_in_round;
    let total_leftover_resource =
            (resources_left as f32 * params.regeneration_factor) as i32;
    let grown_resources_in_round = total_leftover_resource - resources_left;

    // player stats dd
    let player_stats = player_stats_from_moves(player_moves);
    info!("total_leftover_resource : {:?}", total_leftover_resource);

    RoundState {
        resources_left: total_leftover_resource,
        resources_taken: consumed_resources_in_round,
        resources_grown: grown_resources_in_round,
        player_stats,
    }
}

fn get_latest_round(header_hash: HeaderHash) -> ExternResult<(GameRound, EntryHash)> {
    info!("fetching element from DHT");
    debug!("headerhash previous round: {:?}", header_hash.clone());
    let round_element = match get(header_hash, GetOptions::latest())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Round not found".into())),
    };
    debug!("extracting game round from element");
    let last_round: GameRound = entry_from_element_create_or_update(&round_element)?;
    debug!("get latest round: {:#?}", last_round);
    let last_round_entry_hash = round_element
        .header()
        .entry_hash()
        .expect("round element should always have entry hash");
    Ok((last_round, last_round_entry_hash.clone()))
}

// NOTE: game round is always created once players made their moves, so every round is always
// a retrospective of moves made, not created before and updated later
// NOTE: given the retrospective nature, maybe we should call this fn "close current round" or
// "start next round" to avoid adding more confusion
// fn new_game_round(input: GameRoundResultsInput) -> ExternResult<EntryHash> {
//     // validate that player_moves.len() == session.game_params.invited.len(),
//     // otherwise current round isn't complete and we can't create a new one

//     // let state = calculate_round_state
//     // if latest_round not None:
//     //  update existing round entry on the latest_round hash (continuing the update chain)
//     // else:
//     //  create new round entry
//     //  make a link from session -> round
//     // if round is finished or lost:
//     //  update game session state

//     unimplemented!()
// }

// Question: how do we make moves discoverable by the players?
// Option1: make a link from game session / game round to which this move belongs?
//      note: this is where things start to get more complicated with the game round that is
//      only created retrospectively. We will have to manage this duality with link base being
//      either a game session or a game round. But maybe that's not a bad thing? That'll still
//      be a related Holochain entry after all.

// Should retrieve all game moves corresponding to the current round entry (in case of round 0 this
// would actually be a game session entry) and attempt to close the current round by creating it's entry.
// This would solely depend on the amount of moves retrieved being equal to the amount of players in the game

pub fn try_to_close_round(last_round_hash: EntryHash) -> ExternResult<GameRoundInfo> {
    //previous round
    info!("fetching element with previous round from DHT");
    debug!("entry hash previous round: {:?}", last_round_hash.clone());
    let last_round_element = match get(last_round_hash.clone(), GetOptions::latest())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Previous round not found".into())),
    };
    let last_round: GameRound = entry_from_element_create_or_update(&last_round_element)?;

    // TODO optimization: check if new round with next round_num already exists
    // in that case it is not necessary create it again

    // game session
    // fetching element with game session from DHT, trying locally first
    let game_session_element = match get(last_round.session.clone(), GetOptions::latest())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Game session not found".into())),
    };
    let game_session: GameSession = entry_from_element_create_or_update(&game_session_element)?;

    // game moves
    let moves = get_moves_for_round(&last_round_element)?;

    // try to get all moves necessary to close the round
    match finalize_moves(moves, game_session.players.len())? {
        // we get the moves, so we can close the round
        Some(unique_moves) => {
            // TODO: convert Vec<GameMove> into something for nice printing
            let mut moves_info: Vec<(i32, String, AgentPubKey)> = vec![];
            for game_move in &unique_moves {
                moves_info.push((
                    game_move.resources.clone(),
                    "playername".into(),
                    game_move.owner.clone(),
                ));
            }
            info!("all players made their moves: calculating round state");
            let round_state =
                calculate_round_state(&last_round, &game_session.game_params, unique_moves);
            if start_new_round(&game_session, &last_round, &round_state) {
                let hash = create_new_round(
                    &game_session,
                    &last_round,
                    last_round_element.header_address(),
                    &round_state,
                )?;
                return Ok(GameRoundInfo {
                    current_round_entry_hash: Some(hash),
                    prev_round_entry_hash: Some(last_round_hash),
                    game_session_hash: None,
                    resources_left: Some(round_state.resources_left),
                    resources_taken_round: Some(round_state.resources_taken),
                    resources_grown_round: Some(round_state.resources_grown),
                    round_num: last_round.round_num + 1,
                    next_action: "START_NEXT_ROUND".into(),
                    moves: moves_info,
                });
                //round_hash + next action
            } else {
                let hash = crate::game_session::end_game(
                    &game_session,
                    &game_session_element.header_address(),
                    &last_round,
                    last_round_element
                        .header()
                        .entry_hash()
                        .expect("should have entry"),
                    &round_state,
                )?;
                return Ok(GameRoundInfo {
                    current_round_entry_hash: None,
                    prev_round_entry_hash: Some(last_round_hash),
                    game_session_hash: Some(hash),
                    resources_left: Some(round_state.resources_left),
                    resources_taken_round: Some(round_state.resources_taken),
                    resources_grown_round: Some(round_state.resources_grown),
                    round_num: last_round.round_num + 1,
                    next_action: "SHOW_GAME_RESULTS".into(),
                    moves: moves_info,
                });
                //game_session_hash + next action
            }
        }
        // There aren't enough moves yet, so we get nothing and wait
        None => {
            // TODO: fix the value in current_round_entry_hash: Some(last_round_hash)
            return Ok(GameRoundInfo {
                current_round_entry_hash: None,
                prev_round_entry_hash: Some(last_round_hash),
                game_session_hash: Some(
                    game_session_element
                        .header()
                        .entry_hash()
                        .expect("should have entry")
                        .clone(),
                ),
                resources_left: None,
                resources_taken_round: None,
                resources_grown_round: None,
                round_num: last_round.round_num,
                next_action: "WAITING".into(),
                moves: vec![],
                // add anonymous moves list
            });
        }
    };
}

fn start_new_round(
    game_session: &GameSession,
    prev_round: &GameRound,
    round_state: &RoundState,
) -> bool {
    // rounds left to play?
    prev_round.round_num + 1 < game_session.game_params.num_rounds
    // resources not depleted?
        && round_state.resources_left > 0
}

fn create_new_round(
    game_session: &GameSession,
    last_round: &GameRound,
    last_round_header_hash: &HeaderHash,
    round_state: &RoundState,
) -> ExternResult<EntryHash> {
    info!(
        "start new round: updating game round entry. Last_round_num {:?}",
        last_round.round_num
    );
    //update chain from the previous round entry hash and commit an updated version
    let next_round = GameRound::new(
        last_round.round_num + 1,
        last_round.session.clone().into(),
        round_state.resources_left,
        round_state.resources_taken,
        round_state.resources_grown,
    );
    debug!("new round: {:?}", next_round);
    let round_header_hash_update = update_entry(last_round_header_hash.clone(), &next_round)?;
    let round_entry_hash_update = hash_entry(&next_round)?;
    info!("updated round header hash: {:?}", round_header_hash_update);
    debug!("updated round header hash: {:?}", round_header_hash_update);
    info!("signaling player new round has started");
    let signal_payload = SignalPayload {
        game_session_entry_hash: last_round.session.clone(),
        round_entry_hash_update: round_entry_hash_update.clone(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:#?}", game_session.players.clone());

    Ok(round_entry_hash_update)
}

// pub fn current_round_info(game_round_header_hash: HeaderHash) -> ExternResult<GameRoundInfo> {
//     //get latest update for game round
//     let result = get_latest_round(game_round_header_hash)?;
//     let round = result.0;
//     let hash = result.1;
//     let mut round_state: String = "IN_PROGRESS".into();
//     let mut resources: Option<i32> = None;

//     if round.game_moves.len() == 0 {
//         round_state = "FINISHED".into();
//         resources = Some(round.round_state.resource_amount)
//     }
//     let x = GameRoundInfo {
//         round_num: round.round_num,
//         current_round_entry_hash: Some(hash),
//         next_action: round_state,
//         resources_left: resources,
//         game_session_hash: None,
//         moves: vec![],
//     };
//     debug!("Round info: {:?}", x);
//     Ok(x)
// }

pub fn current_round_for_game_code(game_code: String) -> ExternResult<Option<EntryHash>> {
    let anchor = calculate_game_code_anchor_entry_hash(game_code)?;
    let links: Links = get_links(anchor, Some(LinkTag::new("GAME_SESSION")))?;
    let links_vec = links.into_inner();
    debug!("links: {:#?}", &links_vec);

    if links_vec.len() > 0 {
        if links_vec.len() > 1 {
            // TODO find alternative for clone to get len
            return Err(WasmError::Guest(String::from(
                "More than one link from anchor to game session. Should not happen.",
            )));
        }
        // should be only one link
        let link = &links_vec[0];

        debug!("link: {:#?}", link);
        let element: Element = get(link.target.clone(), GetOptions::latest())?
            .ok_or(WasmError::Guest(String::from("Entry not found")))?;
        let game_session: GameSession = element
            .entry()
            .to_app_option()?
            .expect("game session has to exist");
        let game_session_entry_hash: &EntryHash = entry_hash_from_element(&element)?;
        let result = check_agent_is_player_current_session(game_session);
        match result {
            Ok(x) => debug!("player is participant in current game"),
            Err(err) => return Err(err),
        };

        let round_links: Links = get_links(
            game_session_entry_hash.clone(),
            Some(LinkTag::new("GAME_ROUND")),
        )?;
        let round_links_vec = round_links.into_inner();

        if round_links_vec.len() > 0 {
            debug!("links session round: {:#?}", &round_links_vec);
            if round_links_vec.len() > 1 {
                // TODO find alternative for clone to get len
                return Err(WasmError::Guest(String::from(
                    "More than one link from game session to game round. Should not happen.",
                )));
            };
            // should be only one link
            let link = &round_links_vec[0];

            debug!("link session round: {:#?}", &link);
            let element: Element = get(link.target.clone(), GetOptions::latest())?
                .ok_or(WasmError::Guest(String::from("Entry not found")))?;

            //let game_round_entry_hash:&EntryHash = entry_hash_from_element(&element)?;
            return Ok(Some(
                element
                    .header()
                    .entry_hash()
                    .expect("should have entry")
                    .clone(),
            ));
        }
    }

    // in this case the game lead has not yet started the game session
    Ok(None)
}

pub fn validate_create_entry_game_round(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    let game_round: GameRound = data
        .element
        .entry()
        .to_app_option()?
        .ok_or(WasmError::Guest(
            "Trying to validate an entry that's not a GameRound".into(),
        ))?;
    
    if game_round.round_num != 0 {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Can't create GameRound with number {}: number can only be 0",
            game_round.round_num,
        )));
    }
    let game_session = must_get_entry_struct::<GameSession>(game_round.session)?;
    if game_session.status != SessionState::InProgress {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Can't create GameRound for the GameSession {:?} because it's not InProgress",
            game_session,
        )));
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_entry_game_round(
    data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    let game_round: GameRound = data
        .element
        .entry()
        .to_app_option()?
        .ok_or(WasmError::Guest(
            "Trying to validate an entry that's not a GameRound".into(),
        ))?;
    // debug!(
    //     "Validating GameRound update entry {:?}, data: {:?}",
    //     game_round, data
    // );

    let game_session = must_get_entry_struct::<GameSession>(game_round.session)?;
    if game_round.round_num > game_session.game_params.num_rounds {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Can't update GameRound number {} because GameSession only has {} rounds",
            game_round.round_num, game_session.game_params.num_rounds
        )));
    }

    let update_header = data.element.header();

    match update_header {
        Header::Update(update_data) => {
            let prev_entry =
                must_get_entry_struct::<GameRound>(update_data.original_entry_address.clone())?;
            if (prev_entry.round_num + 1) != game_round.round_num {
                return Ok(ValidateCallbackResult::Invalid(format!("Can't update GameRound entry to have round num {}: previous GameRound has num {}", game_round.round_num, prev_entry.round_num)));
            }
        }
        _ => {
            // TODO(e-nastasia): perhaps add there the type of header received, for a more informative error message
            return Ok(ValidateCallbackResult::Invalid(String::from(
                "GameRound's element has the wrong header: expected Update",
            )));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}
