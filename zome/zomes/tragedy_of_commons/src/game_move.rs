use crate::{game_round::{GameRound, RoundState, calculate_round_state}, game_session::{GameScores, GameSession, GameSignal, SignalPayload}, types::ResourceAmount, utils::try_get_and_convert};
use hdk::prelude::*;

#[hdk_entry(id = "game_move", visibility = "public")]
pub struct GameMove {
    pub owner: AgentPubKey,
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

        - TODO: impl validation to make sure move is commited by player who's playing the game
        
*/
#[hdk_extern]
pub fn new_move(input: GameMoveInput) -> ExternResult<HeaderHash>{
    // todo: add guard clauses for empty input
    // todo: calculate agent address
    // todo: create a GameMove entry
    let game_move = GameMove {
        owner: agent_info()?.agent_initial_pubkey,
        resources: input.resource_amount,
        previous_round: input.previous_round.clone(),
    };
    create_entry(&game_move);
    let entry_hash_game_move = hash_entry(&game_move)?;
    
    let header_hash_link= create_link(input.previous_round.clone().into(), 
    entry_hash_game_move.clone(), LinkTag::new("game_move"))?;
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
pub fn try_to_close_round(prev_round_hash: EntryHash) -> ExternResult<EntryHash>{
    let prev_round:GameRound = try_get_and_convert(prev_round_hash.clone())?;
    let game_session:GameSession = try_get_and_convert(prev_round.session.clone())?;    
    let links  = get_links(prev_round_hash, Some(LinkTag::new("game_move")))?;
    let links_vec = links.into_inner();
    if (links_vec.len() < game_session.players.len()){
        let missing_moves_count = game_session.players.len() - links_vec.len();
        return Err(WasmError::Guest(format!("Still waiting on {} players", missing_moves_count)))
    }
    let mut moves:Vec<GameMove> = vec![];
    for l in links_vec {
        let game_move:GameMove = try_get_and_convert(l.target)?;
        moves.push(game_move);
    }
    
    let round_state = calculate_round_state(game_session.game_params, moves);

    if round_state.resource_amount > 0 {
        create_new_round(prev_round.round_num, game_session.clone(), round_state)
    } else {
        end_game(game_session.clone(), round_state)
    }
}

fn create_new_round(prev_round_num:u32, session:GameSession, round_state:RoundState) -> ExternResult<EntryHash> {
    let session_hash = hash_entry(&session)?;
    let round = GameRound{
        round_num: prev_round_num + 1,
        round_state: round_state,
        session: session_hash.clone(),
        previous_round_moves: vec![],
    };
    create_entry(&round)?;
    let entry_hash_round = hash_entry(&round)?;

    // todo send GameSignal: StartNextRound
    let signal_payload = SignalPayload{ 
        // tixel: not sure if we need the full objects or only the hashes or both. The tests will tell...
        game_session: session.clone(),
        game_session_entry_hash: session_hash.clone(),
        previous_round: round,
        previous_round_entry_hash: entry_hash_round.clone(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    remote_signal(signal, session.players.clone())?;
    tracing::debug!("sending signal to {:?}", session.players.clone());

    Ok(entry_hash_round)  
}

fn end_game(session:GameSession, round_state:RoundState) -> ExternResult<EntryHash>{
    let session_hash = hash_entry(&session)?;
    // todo send GameSignal: StartNextRound
    let scores = GameScores{ 
        // tixel: not sure if we need the full objects or only the hashes or both. The tests will tell...
        game_session: session.clone(),
        game_session_entry_hash: session_hash.clone(),
    };
    create_entry(&scores)?;
    let scores_entry_hash = hash_entry(&scores)?;
    let signal = ExternIO::encode(GameSignal::GameOver(scores))?;
    remote_signal(signal, session.players.clone())?;
    tracing::debug!("sending signal to {:?}", session.players.clone());

    Ok(scores_entry_hash)
}

// Retrieves all available game moves made in a certain round, where entry_hash identifies
// base for the links.
fn get_all_round_moves(entry_hash: EntryHash) {
    unimplemented!()
}
