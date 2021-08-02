use crate::game_move::GameMove;
use crate::game_session::{GameParams, GameScores, GameSession, GameSignal, SignalPayload};
use crate::types::{PlayerStats, ReputationAmount, ResourceAmount};
use crate::utils::{
    convert_keys_from_b64, entry_from_element_create_or_update, entry_hash_from_element,
};
use hdk::prelude::*;
use holo_hash::*;
use std::collections::HashMap;

const NO_REPUTATION: ReputationAmount = 0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundState {
    pub resource_amount: ResourceAmount,
    pub player_stats: PlayerStats,
}

#[hdk_entry(id = "game_round", visibility = "public")]
pub struct GameRound {
    pub round_num: u32,
    pub session: EntryHash,
    pub round_state: RoundState,
    pub game_moves: Vec<EntryHash>,
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
        session: EntryHash,
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

/*
validation rules:

- In any game session there's always only one round with the respective round_num
- len of rounds update chain is always <= game_session.params.num_rounds + 1

- validation calculus: validate one round at a time and assume params of previous round
    are already valid
-

TODO: impl validation as:
validate_update_entry_game_round_results -> EntryID


*/

// NOTE: this fn would be used both in validation and when creating game round entries
// so it has to be very lightweight and can not make any DHT queries
pub fn calculate_round_state(params: &GameParams, player_moves: Vec<GameMove>) -> RoundState {
    // todo:
    // calculate round state from the player moves

    // resources
    let consumed_resources_in_round: i32 = player_moves.iter().map(|x| x.resources).sum();
    let total_leftover_resource = params.start_amount - consumed_resources_in_round;

    // player stats
    let mut stats: HashMap<AgentPubKey, (ResourceAmount, ReputationAmount)> = HashMap::new();
    for p in player_moves.iter() {
        let a = p.owner.clone();
        let tuple: (ResourceAmount, ReputationAmount) = (p.resources, NO_REPUTATION);
        stats.insert(a, tuple);
    }
    info!("total_leftover_resource : {:?}", total_leftover_resource);

    RoundState {
        resource_amount: total_leftover_resource,
        player_stats: stats,
    }
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
pub fn try_to_close_round(last_round_hash: HeaderHash) -> ExternResult<HeaderHash> {
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
    let game_session_element = match get(last_round.session.clone(), GetOptions::content())? {
        Some(element) => element,
        None => return Err(WasmError::Guest("Game session not found".into())),
    };
    debug!("extracting game session from element");
    let game_session: GameSession = entry_from_element_create_or_update(&game_session_element)?;

    // game moves
    info!("fetching links to game moves");
    let links = get_links(
        entry_from_element_create_or_update(&last_round_element)?,
        Some(LinkTag::new("game_move")),
    )?;
    let mut moves: Vec<GameMove> = vec![];
    for link in links.into_inner() {
        println!("fetching game move element, trying locally first");
        let game_move_element = match get(link.target.clone(), GetOptions::content())? {
            Some(element) => element,
            None => return Err(WasmError::Guest("Game move not found".into())),
        };
        let game_move: GameMove = entry_from_element_create_or_update(&game_move_element)?;
        moves.push(game_move);
    }

    info!("checking number of moves");
    debug!("moves list #{:?}", moves);
    if &moves.len() < &game_session.players.len() {
        // TODO: implement check to verify that each player has made a single move
        // Since we're not validating that every player has only made one move, we need to make
        // this check here, otherwise game would be broken.
        info!("Cannot close round: wait until all moves are made");
        debug!(
            "number of moves found: #{:?}",
            &game_session.players.len() - &moves.len()
        );
        return Err(WasmError::Guest(
            format!(
                "Still waiting on {} players",
                &game_session.players.len() - &moves.len()
            )
            .into(),
        ));
    };

    info!("all players made their moves: calculating round state");
    let round_state = calculate_round_state(&game_session.game_params, moves);

    // TODO: add check here that we're creating a new round only if
    // it's num is < game.num_rounds, so that we don't accidentally create more rounds
    // than are supposed to be in the game
    if start_new_round(&game_session, &last_round, &round_state) {
        return create_new_round(
            &game_session,
            &last_round,
            last_round_element.header_address(),
            &round_state,
        );
    } else {
        return end_game(
            &game_session,
            &last_round,
            last_round_element.header_address(),
            &round_state,
        );
    }
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
    let round = GameRound {
        round_num: last_round.round_num + 1,
        round_state: round_state.clone(),
        session: last_round.session.clone(),
        game_moves: vec![],
    };
    info!("creating new game round");
    //update chain from the previous round entry hash and commit an updated version
    let round_header_hash_update = update_entry(last_round_header_hash.clone(), &round)?;
    info!("signaling player new round has started");
    let signal_payload = SignalPayload {
        game_session_entry_hash: last_round.session.clone(),
        round_header_hash_update: round_header_hash_update.clone(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:?}", game_session.players.clone());

    Ok(round_header_hash_update)
}

fn end_game(
    game_session: &GameSession,
    last_round: &GameRound,
    last_round_header_hash: &HeaderHash,
    round_state: &RoundState,
) -> ExternResult<HeaderHash> {
    info!("ending game");
    // last_round contains end results
    // so no creates or update are necessary
    // only a signal to all players that game has ended
    // players that miss the signal should have their UI poll GameRound
    // based on that content it can be derive if the game has ended or not
    info!("signaling player game has ended");
    let signal_payload = SignalPayload {
        game_session_entry_hash: last_round.session.clone(),
        round_header_hash_update: last_round_header_hash.clone(),
    };
    let signal = ExternIO::encode(GameSignal::GameOver(signal_payload))?;
    // Since we're storing agent keys as AgentPubKeyB64, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:?}", game_session.players.clone());

    Ok(last_round_header_hash.clone())
}

// Retrieves all available game moves made in a certain round, where entry_hash identifies
// base for the links.
fn get_all_round_moves(round_entry_hash: EntryHash) {
    unimplemented!();
}

#[cfg(test)]
#[rustfmt::skip]   // skipping formatting is needed, because to correctly import fixt we needed "use ::fixt::prelude::*;" which rustfmt does not like

mod tests {
    use crate::types::new_player_stats;

    use super::*;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    use mockall::mock;
    use std::vec;

    #[test]
    fn test_calculate_round_state() {
        let gp = GameParams {
            regeneration_factor: 1.1,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };

        let p1_key = fixt!(AgentPubKey);
        let move1 = GameMove {
            owner: p1_key.clone(),
            previous_round: fixt!(EntryHash),
            resources: 5,
        };

        let p2_key = fixt!(AgentPubKey);
        let move2 = GameMove {
            owner: p2_key.clone(),
            previous_round: fixt!(EntryHash),
            resources: 10,
        };
        let s = calculate_round_state(&gp, vec![move1, move2]);
        assert_eq!(&gp.start_amount - 15, s.resource_amount);

        let stats_p1: (ResourceAmount, ReputationAmount) =
            *s.player_stats.get(&p1_key).unwrap();
        assert_eq!(stats_p1.0, 5);
        assert_eq!(stats_p1.1, 0);

        let stats_p2: (ResourceAmount, ReputationAmount) =
            *s.player_stats.get(&p2_key).unwrap();
        assert_eq!(stats_p2.0, 10);
        assert_eq!(stats_p1.1, 0);
    }
    
    fn test_start_new_round() {

        let game_session = GameSession {
            owner: fixt!(AgentPubKey),
            status: crate::game_session::SessionState::InProgress,
            game_params: GameParams {
                regeneration_factor: 1.1,
                start_amount: 100,
                num_rounds: 3,
                resource_coef: 3,
                reputation_coef: 2,
            },
            players: vec![],
        };
        let prev_round_not_last_round = GameRound {
            round_num: 1,
            session: fixt!(EntryHash),
            round_state: RoundState{ resource_amount: 100, player_stats:HashMap::new() },
            game_moves: vec![],
        };
        let round_state_resources_left = RoundState {
            resource_amount: 10,
            player_stats: HashMap::new(),
        };
        let prev_round_last_round = GameRound {
            round_num: 3,
            session: fixt!(EntryHash),
            round_state: RoundState{ resource_amount: 100, player_stats:HashMap::new() },
            game_moves: vec![],
        };
        let round_state_resources_depleted = RoundState {
            resource_amount: -10,
            player_stats: HashMap::new(),
        };
        let round_state_resources_zero = RoundState {
            resource_amount: 0,
            player_stats: HashMap::new(),
        };

        assert_eq!(true, start_new_round(&game_session, &prev_round_not_last_round,  &round_state_resources_left));
        assert_eq!(false, start_new_round(&game_session, &prev_round_last_round, &round_state_resources_left));
        assert_eq!(true, start_new_round(&game_session, &prev_round_not_last_round, &round_state_resources_depleted));
        assert_eq!(true, start_new_round(&game_session, &prev_round_not_last_round, &round_state_resources_zero));       
    }
}
