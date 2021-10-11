use crate::{game_round::{calculate_round_state, GameRound, RoundState}, game_session::{GameScores, GameSession, GameSignal, SignalPayload}, types::ResourceAmount, utils::{check_agent_is_player_current_session, convert, convert_keys_from_b64, entry_from_element_create_or_update, entry_hash_from_element, must_get_entry_struct, try_get_and_convert}};
use hdk::prelude::holo_hash::hash_type::Agent;
use hdk::prelude::*;
use std::collections::BTreeMap;

pub const GAME_MOVE_LINK_TAG: &str = "GAME_MOVE";

#[hdk_entry(id = "game_move", visibility = "public")]
#[derive(Clone)]
pub struct GameMove {
    pub owner: AgentPubKey,
    // For the very first round this option would be None, because we create game rounds
    // retrospectively. And since all players are notified by the signal when they can make
    // a move, maybe we could pass that value from there, so that every player has it
    // when they're making a move
    pub round: EntryHash,
    pub resources: ResourceAmount,
}
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameMoveInput {
    pub resource_amount: ResourceAmount,
    // NOTE: if we're linking all moves to the round, this can never be None
    // as we'll need a base for the link. Instead moves for the round 0 could be
    // linked directly from the game session.
    pub previous_round: EntryHash,
}

pub fn new_move(
    resource_amount: ResourceAmount,
    round_entry_hash: EntryHash,
) -> ExternResult<HeaderHash> {
    // round
    let game_round_element = match get(round_entry_hash.clone(), GetOptions::content())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Round not found".into())),
    };
    let entry_hash_game_round = entry_hash_from_element(&game_round_element)?.to_owned();

    let game_round: GameRound = game_round_element
        .entry()
        .to_app_option()?
        .to_owned()
        .expect("game round should be known");

    // game session
    let game_session_element = match get(game_round.session.clone(), GetOptions::content())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Round not found".into())),
    };
    let game_session: GameSession = game_session_element
        .entry()
        .to_app_option()?
        .to_owned()
        .expect("game session should be known");

    check_agent_is_player_current_session(game_session);

    // todo: add guard clauses for empty input
    debug!(
        "current round: {:?} amount: {:?}",
        round_entry_hash, resource_amount
    );
    let game_move = GameMove {
        owner: agent_info()?.agent_initial_pubkey,
        resources: resource_amount,
        round: round_entry_hash.clone(),
    };
    create_entry(&game_move);
    let entry_hash_game_move = hash_entry(&game_move)?;

    debug!(
        "link move {:?} to round {:?}",
        &game_move,
        entry_hash_game_round.clone()
    );

    let header_hash_link = create_link(
        entry_hash_game_round,
        entry_hash_game_move.clone(),
        LinkTag::new(String::from(GAME_MOVE_LINK_TAG)),
    )?;
    // note: instead of calling try_to_close_Round right here, we can have a UI make
    // this call for us. This way making a move wouldn't be blocked by the other moves'
    // retrieval process and the process of committing the round entry.
    Ok(header_hash_link)
}

pub fn get_moves_for_round(last_round_element: &Element) -> ExternResult<Vec<GameMove>> {
    info!("fetching links to game moves");
    let links = get_links(
        entry_hash_from_element(last_round_element)?.to_owned(),
        Some(LinkTag::new(String::from(GAME_MOVE_LINK_TAG))),
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
    Ok(moves)
}

/// Consumes list of moves passed to it to finalize them.
/// If every player made at least one move, it returns list of moves which is guaranteed
/// to have (TODO: the earliest) a single move for every player.
/// If there are missing moves, it returns None, since we can't finalize the moves and
/// have to wait for other players instead.
pub fn finalize_moves(
    moves: Vec<GameMove>,
    number_of_players: usize,
) -> ExternResult<Option<Vec<GameMove>>> {
    info!("checking number of moves");
    debug!("moves list #{:?}", moves);
    // Check that at least we have as many moves
    // as there are players in the game
    if moves.len() < number_of_players {
        info!("Cannot close round: wait until all moves are made");
        debug!("number of moves found: #{:?}", moves.len());
        return Ok(None);
    } else {
        // Now that we know we have moves >= num of players, we need
        // to make sure that every player made at least one move, so
        // we're not closing the round without someone's move
        let mut moves_per_player: BTreeMap<AgentPubKey, Vec<GameMove>> = BTreeMap::new();
        for m in moves {
            match moves_per_player.get_mut(&m.owner) {
                Some(mut moves) => moves.push(m),
                // TODO(e-nastasia): cloning owner value seems like a waste, but I think
                // that alternative would be to use lifetimes. Not sure it's worth the
                // readability penalty that we'll incur.
                None => {
                    moves_per_player.insert(m.owner.clone(), vec![m]);
                }
            }
        }
        if moves_per_player.keys().len() < number_of_players {
            info!("Cannot close the round: only {} players made their moves, waiting for total {} players", moves_per_player.keys().len(), number_of_players);
            return Ok(None);
        }
        let mut new_moves = vec![];
        for (owner, move_vec) in moves_per_player {
            // TODO: instead of taking just a [0] move, find the move with the earliest
            // timestamp and use it
            new_moves.push(move_vec[0].clone());
        }
        Ok(Some(new_moves))
    }
}

/*
for the context, here are notes on how we've made decisions about validation rules:
- validate that one player only made one move for any round
    - right now we'll need to run get_links for that, can we avoid it?
    - alternative: get agent activity
        retrieves source chain headers from this agent
        get all headers that are get_link / new entry for game move
        validate that we're not repeating the same move
    - another alternative: avoid strict validation here, instead take first move
        made by agent for any round and use it when calculating
        - NOTE: we'll have vulnerability
        - NOTE: update round closing rules to check that every AGENT made a move
            - upd: this is done in finalize_moves
- validate that moves are made with timestamp >= game session
*/
pub fn validate_create_entry_game_move(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    let game_move: GameMove = entry_from_element_create_or_update(&data.element)?;

    debug!(
        "Validating GameMove create_entry {:?}, data: {:?}",
        game_move, data
    );
    // validate that resources consumed during the move are always positive
    if game_move.resources <= 0 {
        debug!(
            "GameMove {:?} has non-positive resources, INVALID",
            game_move
        );
        return Ok(ValidateCallbackResult::Invalid(format!(
            "GameMove has to have resources >= 0, but it has {}",
            game_move.resources
        )));
    }

    // now we need to retrieve game session via the round header hash saved
    // in the game move entry to verify that player is making a move for the
    // game session they're actually playing
    let game_round = must_get_entry_struct::<GameRound>(game_move.round)?;
    let game_session = must_get_entry_struct::<GameSession>(game_round.session)?;

    if !game_session.players.contains(&game_move.owner) {
        return Ok(ValidateCallbackResult::Invalid(String::from("Can't make a GameMove for this GameSession because move owner isn't in the list of GameSession players")));
    }

    // TODO(e-nastasia): validate that timestamp is later than game_session timestamp

    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_entry_game_move(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Can't update GameMove entry",
    )))
}

pub fn validate_delete_entry_game_move(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Can't delete GameMove entry",
    )))
}
