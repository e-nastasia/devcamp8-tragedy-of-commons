use crate::game_code::GAME_CODES_ANCHOR;
use crate::game_move::GameMove;
use crate::game_session::{
    GameParams, GameScores, GameSession, GameSignal, SessionState, SignalPayload,
};
use crate::types::{PlayerStats, ResourceAmount};
use crate::utils::{
    convert_keys_from_b64, entry_from_element_create_or_update, entry_hash_from_element,
    must_get_header_and_entry,
};
use hdk::prelude::holo_hash::*;
use hdk::prelude::*;
use std::collections::HashMap;
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundState {
    pub resource_amount: ResourceAmount,
    pub player_stats: PlayerStats,
}

#[hdk_entry(id = "game_round", visibility = "public")]
#[derive(Clone, PartialEq, Eq)]
pub struct GameRound {
    pub round_num: u32,
    pub session: HeaderHash,
    pub round_state: RoundState,
    pub game_moves: Vec<EntryHash>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameRoundInfo {
    pub round_num: u32,
    pub resources_left: Option<i32>,
    pub current_round_header_hash: Option<HeaderHash>,
    pub game_session_hash: Option<HeaderHash>,
    pub next_action: String,
    pub moves: Vec<(i32, String, String)>,
}

impl RoundState {
    /// Creates a new RoundState instance with the provided input
    pub fn new(resource_amount: ResourceAmount, player_stats: PlayerStats) -> RoundState {
        RoundState {
            resource_amount,
            player_stats,
        }
    }
}

impl GameRound {
    /// Creates a new GameRound instance with the provided input
    pub fn new(
        round_num: u32,
        session: HeaderHash,
        round_state: RoundState,
        previous_round_moves: Vec<EntryHash>,
    ) -> GameRound {
        GameRound {
            round_num,
            session,
            round_state,
            game_moves: previous_round_moves,
        }
    }
}

// NOTE: this fn would be used both in validation and when creating game round entries
// so it has to be very lightweight and can not make any DHT queries
pub fn calculate_round_state(params: &GameParams, player_moves: Vec<GameMove>) -> RoundState {
    // todo:
    // calculate round state from the player moves

    // resources
    let consumed_resources_in_round: i32 = player_moves.iter().map(|x| x.resources).sum();
    let total_leftover_resource = params.start_amount - consumed_resources_in_round;

    // player stats dd
    let mut stats: HashMap<AgentPubKey, ResourceAmount> = HashMap::new();
    for p in player_moves.iter() {
        let a = p.owner.clone();
        let r: ResourceAmount = p.resources;
        stats.insert(a, r);
    }
    info!("total_leftover_resource : {:?}", total_leftover_resource);

    RoundState {
        resource_amount: total_leftover_resource,
        player_stats: stats,
    }
}

fn get_latest_round(header_hash: HeaderHash) -> ExternResult<(GameRound, HeaderHash)> {
    info!("fetching element from DHT");
    debug!(
        "headerhash previous round: {:?}",
        HeaderHashB64::from(header_hash.clone())
    );
    let round_element = match get(header_hash, GetOptions::latest())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Round not found".into())),
    };
    debug!("extracting game round from element");
    let last_round: GameRound = entry_from_element_create_or_update(&round_element)?;
    debug!("get latest round: {:?}", last_round);
    Ok((last_round, round_element.header_address().clone()))
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

pub fn try_to_close_round(last_round_hash: HeaderHash) -> ExternResult<GameRoundInfo> {
    //previous round
    info!("fetching element with previous round from DHT");
    debug!(
        "headerhash previous round: {:?}",
        HeaderHashB64::from(last_round_hash.clone())
    );
    let last_round_element = match get(last_round_hash.clone(), GetOptions::latest())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Previous round not found".into())),
    };
    debug!("extracting game round from element");
    let last_round: GameRound = entry_from_element_create_or_update(&last_round_element)?;

    // game session
    info!("fetching element with game session from DHT, trying locally first");
    let game_session_element = match get(last_round.session.clone(), GetOptions::latest())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Game session not found".into())),
    };
    debug!("extracting game session from element");
    let game_session: GameSession = entry_from_element_create_or_update(&game_session_element)?;

    // game moves
    info!("fetching links to game moves");
    let links = get_links(
        entry_hash_from_element(&last_round_element)?.to_owned(),
        Some(LinkTag::new("GAME_MOVE")),
    )?;
    let mut moves: Vec<GameMove> = vec![];
    for link in links.into_inner() {
        debug!("fetching game move element, trying locally first");
        let game_move_element = match get(link.target.clone(), GetOptions::latest())? {
            Some(element) => element,
            None => return Err(WasmError::Guest("Game move not found".into())),
        };
        let game_move: GameMove = entry_from_element_create_or_update(&game_move_element)?;
        moves.push(game_move);
    }

    let b = missing_moves(&moves, game_session.players.len());
    if (b) {
        let mut anonymous_moves_info: Vec<(i32, String, String)> = vec![];
        for game_move in &moves {
            anonymous_moves_info.push((
                -1,
                "playername".into(),
                HoloHashB64::from(game_move.owner.clone()).to_string(),
            ));
        }

        return Ok(GameRoundInfo {
            current_round_header_hash: Some(last_round_hash),
            game_session_hash: Some(game_session_element.header_address().clone()),
            resources_left: Some(last_round.round_state.resource_amount),
            round_num: last_round.round_num,
            next_action: "WAITING".into(),
            moves: vec![(12, "Bobby".into(), "msqljfmsqfdkmqlkdsf".into())],
            // add anonymous moves list
        });
        // existing hashes + next action
    }

    let mut moves_info: Vec<(i32, String, String)> = vec![];
    for game_move in &moves {
        moves_info.push((
            game_move.resources.clone(),
            "playername".into(),
            HoloHashB64::from(game_move.owner.clone()).to_string(),
        ));
    }
    info!("all players made their moves: calculating round state");
    let round_state = calculate_round_state(&game_session.game_params, moves);

    // TODO: add check here that we're creating a new round only if
    // it's num is < game.num_rounds, so that we don't accidentally create more rounds
    // than are supposed to be in the game
    if start_new_round(&game_session, &last_round, &round_state) {
        let hash = create_new_round(
            &game_session,
            &last_round,
            last_round_element.header_address(),
            &round_state,
        )?;
        Ok(GameRoundInfo {
            current_round_header_hash: Some(hash),
            game_session_hash: None,
            resources_left: Some(round_state.resource_amount),
            round_num: last_round.round_num + 1,
            next_action: "START_NEXT_ROUND".into(),
            moves: moves_info,
        })
        //round_hash + next action
    } else {
        let hash = crate::game_session::end_game(
            &game_session,
            &game_session_element.header_address(),
            &last_round,
            last_round_element.header_address(),
            &round_state,
        )?;
        Ok(GameRoundInfo {
            current_round_header_hash: None,
            game_session_hash: Some(hash),
            resources_left: Some(round_state.resource_amount),
            round_num: last_round.round_num + 1,
            next_action: "SHOW_GAME_RESULTS".into(),
            moves: moves_info,
        })
        //game_session_hash + next action
    }
}

fn missing_moves(moves: &Vec<GameMove>, number_of_players: usize) -> bool {
    info!("checking number of moves");
    debug!("moves list #{:?}", moves);
    if moves.len() < number_of_players {
        // TODO: implement check to verify that each player has made a single move
        // Since we're not validating that every player has only made one move, we need to make
        // this check here, otherwise game would be broken.
        info!("Cannot close round: wait until all moves are made");
        debug!("number of moves found: #{:?}", moves.len());
        return true;
    };
    false
}

fn start_new_round(
    game_session: &GameSession,
    prev_round: &GameRound,
    round_state: &RoundState,
) -> bool {
    // rounds left to play?
    prev_round.round_num + 1 < game_session.game_params.num_rounds
    // resources not depleted?
        && round_state.resource_amount > 0
}

fn create_new_round(
    game_session: &GameSession,
    last_round: &GameRound,
    last_round_header_hash: &HeaderHash,
    round_state: &RoundState,
) -> ExternResult<HeaderHash> {
    info!(
        "start new round: updating game round entry. Last_round_num {:?}",
        last_round.round_num
    );
    //update chain from the previous round entry hash and commit an updated version
    let next_round = GameRound {
        round_num: last_round.round_num + 1,
        round_state: round_state.clone(),
        session: last_round.session.clone().into(),
        game_moves: vec![],
    };
    debug!("new round: {:?}", next_round);
    let round_header_hash_update = update_entry(last_round_header_hash.clone(), &next_round)?;
    debug!("updated round header hash: {:?}", round_header_hash_update);
    info!("signaling player new round has started");
    let signal_payload = SignalPayload {
        game_session_header_hash: HeaderHashB64::from(last_round.session.clone()),
        round_header_hash_update: round_header_hash_update.clone().into(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:?}", game_session.players.clone());

    Ok(round_header_hash_update)
}

pub fn current_round_info(game_round_header_hash: HeaderHash) -> ExternResult<GameRoundInfo> {
    //get latest update for game round
    let result = get_latest_round(game_round_header_hash)?;
    let round = result.0;
    let hash = result.1;
    let mut round_state: String = "IN_PROGRESS".into();
    let mut resources: Option<i32> = None;

    if round.game_moves.len() == 0 {
        round_state = "FINISHED".into();
        resources = Some(round.round_state.resource_amount)
    }
    let x = GameRoundInfo {
        round_num: round.round_num,
        current_round_header_hash: Some(hash),
        next_action: round_state,
        resources_left: resources,
        game_session_hash: None,
        moves: vec![],
    };
    debug!("Round info: {:?}", x);
    Ok(x)
}

pub fn current_round_for_game_code(game_code: String) -> ExternResult<Option<HeaderHash>> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), game_code.clone())?;
    let links: Links = get_links(anchor, Some(LinkTag::new("GAME_SESSION")))?;
    let links_vec = links.into_inner();
    debug!("links: {:?}", &links_vec);

    if links_vec.len() > 0 {
        if links_vec.len() > 1 {
            // TODO find alternative for clone to get len
            return Err(WasmError::Guest(String::from(
                "More than one link from anchor to game session. Should not happen.",
            )));
        }
        // should be only one link
        let link = &links_vec[0];

        debug!("link: {:?}", link);
        let element: Element = get(link.target.clone(), GetOptions::latest())?
            .ok_or(WasmError::Guest(String::from("Entry not found")))?;

        let game_session_entry_hash: &EntryHash = entry_hash_from_element(&element)?;

        let round_links: Links = get_links(
            game_session_entry_hash.clone(),
            Some(LinkTag::new("GAME_ROUND")),
        )?;
        let round_links_vec = round_links.into_inner();

        if round_links_vec.len() > 0 {
            debug!("links session round: {:?}", &round_links_vec);
            if round_links_vec.len() > 1 {
                // TODO find alternative for clone to get len
                return Err(WasmError::Guest(String::from(
                    "More than one link from game session to game round. Should not happen.",
                )));
            };
            // should be only one link
            let link = &round_links_vec[0];

            debug!("link session round: {:?}", &link);
            let element: Element = get(link.target.clone(), GetOptions::latest())?
                .ok_or(WasmError::Guest(String::from("Entry not found")))?;

            //let game_round_entry_hash:&EntryHash = entry_hash_from_element(&element)?;
            return Ok(Some(element.header_address().clone()));
        }
    }

    // in this case the game lead has not yet started the game session
    Ok(None)
}

// Retrieves all available game moves made in a certain round, where entry_hash identifies
// base for the links.
fn get_all_round_moves(round_entry_hash: EntryHash) {
    unimplemented!();
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
    debug!(
        "Validating GameRound update entry {:?}, data: {:?}",
        game_round, data
    );
    let game_session = must_get_header_and_entry::<GameSession>(game_round.session)?;
    if game_round.round_num > game_session.game_params.num_rounds {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Can't create GameRound number {} because GameSession only has {} rounds",
            game_round.round_num, game_session.game_params.num_rounds
        )));
    }
    debug!(
        "Validated that round_num {} is correct",
        game_round.round_num
    );

    let update_header = data.element.header();
    debug!("Matching update_header {:?}", update_header);
    match update_header {
        Header::Update(update_data) => {
            debug!("Getting prev GameRound entry");
            let prev_entry = must_get_header_and_entry::<GameRound>(
                update_data.original_header_address.clone(),
            )?;
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
