use crate::{
    game_round::{calculate_round_state, GameRound, RoundState},
    game_session::{GameScores, GameSession, GameSignal, SignalPayload},
    types::ResourceAmount,
    utils::{convert_keys_from_b64, entry_hash_from_element, try_get_and_convert},
};
use hdk::prelude::*;
use holo_hash::AgentPubKeyB64;

#[hdk_entry(id = "game_move", visibility = "public")]
pub struct GameMove {
    pub owner: AgentPubKey,
    // For the very first round this option would be None, because we create game rounds
    // retrospectively. And since all players are notified by the signal when they can make
    // a move, maybe we could pass that value from there, so that every player has it
    // when they're making a move
    pub round: HeaderHash,
    pub resources: ResourceAmount,
}
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameMoveInput {
    pub resource_amount: ResourceAmount,
    // NOTE: if we're linking all moves to the round, this can never be None
    // as we'll need a base for the link. Instead moves for the round 0 could be
    // linked directly from the game session.
    pub previous_round: HeaderHash,
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
        owner: agent_info()?.agent_initial_pubkey,
        resources: resource_amount,
        round: round_header_hash.clone(),
    };
    create_entry(&game_move);
    let entry_hash_game_move = hash_entry(&game_move)?;

    let game_round_element = match get(round_header_hash.clone(), GetOptions::content())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Round not found".into())),
    };

    let header_hash_link = create_link(
        entry_hash_from_element(&game_round_element)?.to_owned(),
        entry_hash_game_move.clone(),
        LinkTag::new("game_move"),
    )?;
    // todo: (if we're making a link from round to move) make a link round -> move
    // note: instead of calling try_to_close_Round right here, we can have a UI make
    // this call for us. This way making a move wouldn't be blocked by the other moves'
    // retrieval process and the process of commiting the round entry.
    Ok(header_hash_link.into())
}
