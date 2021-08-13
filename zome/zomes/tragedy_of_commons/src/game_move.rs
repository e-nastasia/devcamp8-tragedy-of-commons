use crate::{
    game_round::{calculate_round_state, GameRound, RoundState},
    game_session::{GameScores, GameSession, GameSignal, SignalPayload},
    types::ResourceAmount,
    utils::{convert_keys_from_b64, try_get_and_convert},
};
use hdk::prelude::*;
use holo_hash::AgentPubKeyB64;

#[hdk_entry(id = "game_move", visibility = "public")]
pub struct GameMove {
    pub owner: AgentPubKeyB64,
    // For the very first round this option would be None, because we create game rounds
    // retrospectively. And since all players are notified by the signal when they can make
    // a move, maybe we could pass that value from there, so that every player has it
    // when they're making a move
    pub previous_round: EntryHash,
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

/*
validation rules:
    - TODO: impl validation to make sure move is commited by player who's playing the game

for the context, here are notes on how we've made this decision:
- validate that one player only made one move for any round
    - right now we'll need to run get_links for that, can we avoid it?
    - alternative: get agent activity
        retrieves source chain headers from this agent
        get all headers that are get_link / new entry for game move
        validate that we're not repeating the same move

        validate that moves are made with timestamp >= game session
    - another alternative: avoid strict validation here, instead take first move
        made by agent for any round and use it when calculating
        - NOTE: we'll have vulnerability
        - NOTE: update round closing rules to check that every AGENT made a move
*/
pub fn new_move(input: GameMoveInput) -> ExternResult<HeaderHash> {
    // todo: add guard clauses for empty input
    let game_move = GameMove {
        owner: AgentPubKeyB64::from(agent_info()?.agent_initial_pubkey),
        resources: input.resource_amount,
        previous_round: input.previous_round.clone(),
    };
    create_entry(&game_move);
    let entry_hash_game_move = hash_entry(&game_move)?;

    let header_hash_link = create_link(
        input.previous_round.clone().into(),
        entry_hash_game_move.clone(),
        LinkTag::new("game_move"),
    )?;
    // todo: (if we're making a link from round to move) make a link round -> move
    // note: instead of calling try_to_close_Round right here, we can have a UI make
    // this call for us. This way making a move wouldn't be blocked by the other moves'
    // retrieval process and the process of commiting the round entry.
    Ok(header_hash_link.into())
}

// Question: how do we make moves discoverable by the players?
// Option1: make a link from game session / game round to which this move belongs?
//      note: this is where things start to get more complicated with the game round that is
//      only created retrospectively. We will have to manage this duality with link base being
//      either a game session or a game round. But maybe that's not a bad thing? That'll still
//      be a related Holochain entry after all.

// Should retrieve all game moves corresponding to the current round entry (in case of round 0 this
// would actually be a game session entry) and attempt to close the current round by creating it's entry.
// This would solely depend on the amount of moves retrieved being equal to the amount of players in the game
#[hdk_extern]
pub fn try_to_close_round(prev_round_hash: EntryHash) -> ExternResult<EntryHash> {
    let prev_round: GameRound = try_get_and_convert(prev_round_hash.clone())?;
    let game_session: GameSession = try_get_and_convert(prev_round.session.clone())?;
    // TODO: refactor getting moves from the previous round into it's own fn get_all_round_moves
    let links = get_links(prev_round_hash, Some(LinkTag::new("game_move")))?;
    let links_vec = links.into_inner();
    // TODO: implement check to verify that each player has made a single move
    // Since we're not validating that every player has only made one move, we need to make
    // this check here, otherwise game would be broken.
    if (links_vec.len() < game_session.players.len()) {
        let missing_moves_count = game_session.players.len() - links_vec.len();
        return Err(WasmError::Guest(format!(
            "Still waiting on {} players",
            missing_moves_count
        )));
    }
    let mut moves: Vec<GameMove> = vec![];
    for l in links_vec {
        let game_move: GameMove = try_get_and_convert(l.target)?;
        moves.push(game_move);
    }

    let round_state = calculate_round_state(game_session.game_params, moves);

    // TODO: add check here that we're creating a new round only if
    // it's num is < game.num_rounds, so that we don't accidentally create more rounds
    // than are supposed to be in the game
    if round_state.resource_amount > 0 {
        create_new_round(prev_round.round_num, game_session.clone(), round_state)
    } else {
        end_game(game_session.clone(), round_state)
    }
}

// TODO: refactor this fn signature to accept hash of the previous round
fn create_new_round(
    prev_round_num: u32,
    session: GameSession,
    round_state: RoundState,
) -> ExternResult<EntryHash> {
    let session_hash = hash_entry(&session)?;
    // TODO: instead of creating a new entry, we should continue the update chain
    // from the previous round entry hash and commit an updated version
    let round = GameRound {
        round_num: prev_round_num + 1,
        round_state: round_state,
        session: session_hash.clone(),
        previous_round_moves: vec![],
    };
    create_entry(&round)?;
    let entry_hash_round = hash_entry(&round)?;

    let signal_payload = SignalPayload {
        // tixel: not sure if we need the full objects or only the hashes or both. The tests will tell...
        game_session: session.clone(),
        game_session_entry_hash: session_hash.clone(),
        previous_round: round,
        previous_round_entry_hash: entry_hash_round.clone(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    // Since we're storing agent keys as AgentPubKeyB64, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, convert_keys_from_b64(session.players.clone()))?;
    tracing::debug!("sending signal to {:?}", session.players.clone());

    Ok(entry_hash_round)
}

fn end_game(session: GameSession, round_state: RoundState) -> ExternResult<EntryHash> {
    let session_hash = hash_entry(&session)?;
    let scores = GameScores {
        // tixel: not sure if we need the full objects or only the hashes or both. The tests will tell...
        game_session: session.clone(),
        game_session_entry_hash: session_hash.clone(),
    };
    create_entry(&scores)?;
    let scores_entry_hash = hash_entry(&scores)?;

    // TODO: update GameSession entry to set it's state to closed

    let signal = ExternIO::encode(GameSignal::GameOver(scores))?;
    // Since we're storing agent keys as AgentPubKeyB64, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, convert_keys_from_b64(session.players.clone()))?;
    tracing::debug!("sending signal to {:?}", session.players.clone());

    Ok(scores_entry_hash)
}

// Retrieves all available game moves made in a certain round, where entry_hash identifies
// base for the links.
fn get_all_round_moves(round_entry_hash: EntryHash) {
    unimplemented!();
}
