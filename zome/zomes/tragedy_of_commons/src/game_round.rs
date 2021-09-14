use crate::game_move::GameMove;
use crate::game_session::{
    GameParams, GameScores, GameSession, GameSignal, SessionState, SignalPayload,
};
use crate::game_code::GAME_CODES_ANCHOR;
use crate::types::{PlayerStats, ResourceAmount};
use crate::utils::{
    convert_keys_from_b64, entry_from_element_create_or_update, entry_hash_from_element,
    must_get_header_and_entry,
};
use hdk::prelude::*;
use holo_hash::*;
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
    pub current_round_header_hash: HeaderHash,
    pub current_round_state: String,
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
        entry_hash_from_element(&last_round_element)?.to_owned(),
        Some(LinkTag::new("game_move")),
    )?;
    let mut moves: Vec<GameMove> = vec![];
    for link in links.into_inner() {
        debug!("fetching game move element, trying locally first");
        let game_move_element = match get(link.target.clone(), GetOptions::content())? {
            Some(element) => element,
            None => return Err(WasmError::Guest("Game move not found".into())),
        };
        let game_move: GameMove = entry_from_element_create_or_update(&game_move_element)?;
        moves.push(game_move);
    }

    let b = missing_moves(&moves, game_session.players.len());
    if (b) {
        return Err(WasmError::Guest(
            format!(
                "Still waiting on {} player(s)",
                game_session.players.len() - &moves.len()
            )
            .into(),
        ));
    }
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
            &game_session_element.header_address(),
            &last_round,
            last_round_element.header_address(),
            &round_state,
        );
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
        debug!(
            "number of moves found: #{:?}",
            number_of_players - moves.len()
        );
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
    info!("start new round: updating game round entry");
    //update chain from the previous round entry hash and commit an updated version
    let next_round = GameRound {
        round_num: last_round.round_num + 1,
        round_state: round_state.clone(),
        session: last_round.session.clone().into(),
        game_moves: vec![],
    };
    let round_header_hash_update = update_entry(last_round_header_hash.clone(), &next_round)?;
    debug!("updated round header hash: {:?}", round_header_hash_update);
    info!("signaling player new round has started");
    let signal_payload = SignalPayload {
        game_session_header_hash: last_round.session.clone().into(),
        round_header_hash_update: round_header_hash_update.clone().into(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:?}", game_session.players.clone());

    Ok(round_header_hash_update)
}

fn end_game(
    game_session: &GameSession,
    game_session_header_hash: &HeaderHash,
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

    info!("updating game session: setting finished state and adding player stats");
    //update chain for game session entry
    let game_session_update = GameSession {
        owner: game_session.owner.clone(),
        status: SessionState::Finished,
        game_params: game_session.game_params.clone(),
        players: game_session.players.clone(),
        scores: round_state.player_stats.clone(),
    };
    let game_session_header_hash_update =
        update_entry(game_session_header_hash.clone(), &game_session_update)?;
    debug!(
        "updated game session header hash: {:?}",
        game_session_header_hash_update.clone()
    );

    info!("signaling player game has ended");
    let signal_payload = SignalPayload {
        game_session_header_hash: game_session_header_hash_update.clone().into(),
        round_header_hash_update: last_round_header_hash.clone().into(),
    };
    let signal = ExternIO::encode(GameSignal::GameOver(signal_payload))?;
    // Since we're storing agent keys as AgentPubKeyB64, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, game_session.players.clone())?;
    debug!("sending signal to {:?}", game_session.players.clone());

    Ok(game_session_header_hash_update.clone())
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
        current_round_header_hash: hash,
        current_round_state: round_state,
        resources_left: resources,
    };
    Ok(x)
}

pub fn current_round_for_game_code(game_code: String) -> ExternResult<Option<EntryHash>> {
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

        let game_session_header_hash: &EntryHash = entry_hash_from_element(&element)?;

        let round_links: Links = get_links(
            game_session_header_hash.clone(),
            Some(LinkTag::new("GAME_ROUND")),
        )?;
        let round_links_vec = round_links.into_inner();

        if round_links_vec.len() > 0 {
            if round_links_vec.len() > 1 {
                // TODO find alternative for clone to get len
                return Err(WasmError::Guest(String::from(
                    "More than one link from game session to game round. Should not happen.",
                )));
            };
            // should be only one link
            let link = &round_links_vec[0];
            let element: Element = get(link.target.clone(), GetOptions::latest())?
                .ok_or(WasmError::Guest(String::from("Entry not found")))?;

            let game_round_entry_hash: &EntryHash = entry_hash_from_element(&element)?;
            return Ok(Some(game_round_entry_hash.clone()));
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
    let game_session = must_get_header_and_entry::<GameSession>(game_round.session)?;
    if game_round.round_num > game_session.game_params.num_rounds {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Can't create GameRound number {} because GameSession only has {} rounds",
            game_round.round_num, game_session.game_params.num_rounds
        )));
    }

    let update_header = data.element.header();

    match update_header {
        Header::Update(update_data) => {
            let prev_entry =
                must_get_header_and_entry::<GameRound>(update_data.prev_header.clone())?;
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

#[cfg(test)]
#[rustfmt::skip]   // skipping formatting is needed, because to correctly import fixt we needed "use ::fixt::prelude::*;" which rustfmt does not like

mod tests {
    use crate::{types::new_player_stats, utils::enable_tracing};

    use super::*;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    use ::mockall::mock;
    use std::vec;
    use ::holochain_types::prelude::ElementFixturator;


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
            round: fixt!(HeaderHash),
            resources: 5,
        };

        let p2_key = fixt!(AgentPubKey);
        let move2 = GameMove {
            owner: p2_key.clone(),
            round: fixt!(HeaderHash),
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
            scores: PlayerStats::new(),
        };
        let prev_round_not_last_round = GameRound {
            round_num: 1,
            session: fixt!(HeaderHash),
            round_state: RoundState{ resource_amount: 100, player_stats:HashMap::new() },
            game_moves: vec![],
        };
        let round_state_resources_left = RoundState {
            resource_amount: 10,
            player_stats: HashMap::new(),
        };
        let prev_round_last_round = GameRound {
            round_num: 3,
            session: fixt!(HeaderHash),
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

    #[test]
    // to run just this test =>   RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib -- game_round::tests::test_try_to_close_round_fails_not_enough_moves --exact --nocapture
    fn test_try_to_close_round_fails_not_enough_moves() {
        enable_tracing(tracing::Level::DEBUG);

        info!("closing round should fail because only one of two players has made a move.");
        // mock agent info
        let agent_pubkey_alice = fixt!(AgentPubKey);
        let agent_pubkey_bob = fixt!(AgentPubKey);
        let players = vec![agent_pubkey_alice.clone(), agent_pubkey_bob.clone()];

        let mut mock_hdk = hdk::prelude::MockHdkT::new();

        // mock game session element
        let game_params = GameParams {
            regeneration_factor: 1.0,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };
        let game_session = GameSession {
            owner: agent_pubkey_alice.clone(),
           // status: SessionState::InProgress,
            game_params,
            players: players.clone(),
            status: crate::game_session::SessionState::InProgress,
            scores: PlayerStats::new(),
        };

        let mut element_with_game_session: Element = fixt!(Element);
        let create_header_1 = fixt!(Create);
        *element_with_game_session.as_header_mut() = Header::Create(create_header_1.clone());
        *element_with_game_session.as_entry_mut() = ElementEntry::Present(game_session.clone().try_into().unwrap());
        let game_session_header_hash_closure = element_with_game_session.header_address().clone();
        debug!("game session header hash: {:?}", element_with_game_session.header_address().clone());

        let game_round_one = GameRound {
            round_num: 1,
            round_state: RoundState {player_stats: new_player_stats(&players), resource_amount: 100},
            session: element_with_game_session.header_address().clone(),
            game_moves: vec![],
        };

        
        let mut element_with_game_round: Element = fixt!(Element);
        let create_header_2 = fixt!(Create);
        *element_with_game_round.as_header_mut() = Header::Create(create_header_2.clone());
        *element_with_game_round.as_entry_mut() = ElementEntry::Present(game_round_one.clone().try_into().unwrap());
        debug!("game round header hash: {:?}", element_with_game_round.header_address().clone());


        // game move
        let game_move_alice = GameMove {
            owner: agent_pubkey_alice.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 10,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_alice = fixt!(Element);
        let create_header_3 = fixt!(Create);
        *element_with_game_move_alice.as_header_mut() = Header::Create(create_header_3.clone());
        *element_with_game_move_alice.as_entry_mut() =
            ElementEntry::Present(game_move_alice.try_into().unwrap());
        let move_alice_round1_entry_hash = Header::Create(create_header_3.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_alice_header_hash_closure = element_with_game_move_alice.header_address().clone();
        debug!("game move header hash: {:?}", element_with_game_move_alice.header_address().clone());

        // link
        let move_alice_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_alice_round1 = Link {
            target: move_alice_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_alice_round1_link_header_hash.into(),
        };
        let game_moves: Links = vec![link_to_move_alice_round1].into();




        let header_hash_closure = element_with_game_round.header_address().clone();

        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_round.header_address().clone().into(),
            GetOptions::latest(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_round.clone())]));
        
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_session.header_address().clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_session.clone())]));


        mock_hdk
        .expect_get_links()
        .times(1)
        .return_once(move |_| Ok(vec![game_moves]));
        
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_alice_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_alice)]));

        // let header_hash_final_round = fixt!(HeaderHash);
        
        hdk::prelude::set_hdk(mock_hdk);
        let result = try_to_close_round(header_hash_closure.into());
        let err = result.err().unwrap();
        info!("{:?}", err);
        match err {
            WasmError::Guest(x) => assert_eq!(x, "Still waiting on 1 player(s)"),
            _ => assert_eq!(true, false),
        }
    }
    
    #[test]
    fn test_try_to_close_round_success_create_next_round() {
        enable_tracing(tracing::Level::DEBUG);
        // mock agent info
        let agent_pubkey_alice = fixt!(AgentPubKey);
        let agent_pubkey_bob = fixt!(AgentPubKey);
        
        let mut mock_hdk = hdk::prelude::MockHdkT::new();
        
        debug!("prepare game session element");
        // mock game session element
        let game_params = GameParams {
            regeneration_factor: 1.0,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };
        let players = vec![agent_pubkey_alice.clone(), agent_pubkey_bob.clone()];
        let game_session = GameSession {
            owner: agent_pubkey_alice.clone(),
            // status: SessionState::InProgress,
            game_params,
            players: vec![agent_pubkey_alice.clone(), agent_pubkey_bob.clone()],
            status: crate::game_session::SessionState::InProgress,
            scores: PlayerStats::new(),
        };
        let mut element_with_game_session: Element = fixt!(Element);
        let create_header_1 = fixt!(Create);
        *element_with_game_session.as_header_mut() = Header::Create(create_header_1.clone());
        *element_with_game_session.as_entry_mut() = ElementEntry::Present(game_session.clone().try_into().unwrap());
        let game_session_header_hash_closure = element_with_game_session.header_address().clone();
        debug!("game session header hash: {:?}", element_with_game_session.header_address().clone());

        // Game round one
        let game_round_zero = GameRound {
            round_num: 0,
            round_state: RoundState {
                resource_amount: 100,
                player_stats: new_player_stats(&players),
            },
            session: element_with_game_session.header_address().clone(),
            game_moves: vec![],
        };

        debug!("prepare game round element");
        let mut element_with_game_round: Element = fixt!(Element);
        let create_header_2 = fixt!(Create);
        let game_round_header_hash_closure = element_with_game_round.header_address().clone();
        *element_with_game_round.as_header_mut() = Header::Create(create_header_2.clone());
        *element_with_game_round.as_entry_mut() = ElementEntry::Present(game_round_zero.clone().try_into().unwrap());
        debug!("game round header hash: {:?}", element_with_game_round.header_address().clone());

        // Game round one - update
        let game_round_zero_update_to_round_one = GameRound {
            round_num: 1,
            round_state: RoundState {
                resource_amount: 100,
                player_stats: new_player_stats(&players),
            },
            session: element_with_game_session.header_address().clone(),
            game_moves: vec![],
        };

        debug!("prepare game round element - update");
        let mut element_with_game_round_update: Element = fixt!(Element);
        let update_header_1 = fixt!(Update);
        let game_round_update_header_hash_closure = element_with_game_round_update.header_address().clone();
        *element_with_game_round_update.as_header_mut() = Header::Update(update_header_1.clone());
        *element_with_game_round_update.as_entry_mut() = ElementEntry::Present(game_round_zero_update_to_round_one.clone().try_into().unwrap());
        debug!("game round update header hash: {:?}", element_with_game_round_update.header_address().clone());

        // game move alice
        debug!("prepare game move element alice");
        let game_move_alice = GameMove {
            owner: agent_pubkey_alice.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 10,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_alice = fixt!(Element);
        let create_header_3 = fixt!(Create);
        *element_with_game_move_alice.as_header_mut() = Header::Create(create_header_3.clone());
        *element_with_game_move_alice.as_entry_mut() =
            ElementEntry::Present(game_move_alice.try_into().unwrap());
        let move_alice_round1_entry_hash = Header::Create(create_header_3.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_alice_header_hash_closure = element_with_game_move_alice.header_address().clone();


        // game move bob
        debug!("prepare game move element bob");
        let game_move_bob = GameMove {
            owner: agent_pubkey_bob.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 10,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_bob = fixt!(Element);
        let create_header_4 = fixt!(Create);
        *element_with_game_move_bob.as_header_mut() = Header::Create(create_header_4.clone());
        *element_with_game_move_bob.as_entry_mut() =
            ElementEntry::Present(game_move_bob.try_into().unwrap());
        let move_bob_round1_entry_hash = Header::Create(create_header_4.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_bob_header_hash_closure = element_with_game_move_bob.header_address().clone();


        // links
        debug!("prepare game move links");
        let move_alice_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_alice_round1 = Link {
            target: move_alice_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_alice_round1_link_header_hash.into(),
        };

        let move_bob_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_bob_round1 = Link {
            target: move_bob_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_bob_round1_link_header_hash.into(),
        };
        let game_moves: Links = vec![link_to_move_alice_round1, link_to_move_bob_round1].into();


        let header_hash_closure = element_with_game_round.header_address().clone();

        debug!("mock get game round");
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_round.header_address().clone().into(),
            GetOptions::latest(),
            )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_round.clone())]));

        debug!("mock get game session");
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_session.header_address().clone().into(),
            GetOptions::content(),
            )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_session.clone())]));

        debug!("mock get links");
        mock_hdk
            .expect_get_links()
            .times(1)
            .return_once(move |_| Ok(vec![game_moves]));
        
        debug!("mock get game move alice");   
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_alice_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_alice)]));

        debug!("mock get game move bob");   
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_bob_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_bob)]));

        debug!("mock update game round chain to contain game round 1");
        let game_round_1_header_hash = fixt!(HeaderHash);
        let game_round_1_header_hash_closure = game_round_1_header_hash.clone();
        mock_hdk
            .expect_update()
            .times(1)
            .return_once(move |_| Ok(game_round_1_header_hash_closure.clone()));
        
        mock_hdk
            .expect_remote_signal()
            .times(1)
            .return_once(move |_| Ok(()));          
        
        hdk::prelude::set_hdk(mock_hdk);
        let result = try_to_close_round(game_round_header_hash_closure.clone());
        assert_eq!(result.unwrap(), game_round_1_header_hash);
    }

    #[test]
    fn test_try_to_close_round_success_end_game_resources_depleted(){
        enable_tracing(tracing::Level::DEBUG);
        // mock agent info
        let agent_pubkey_alice = fixt!(AgentPubKey);
        let agent_pubkey_bob = fixt!(AgentPubKey);
        let players = vec![agent_pubkey_alice.clone(), agent_pubkey_bob.clone()];
        
        let mut mock_hdk = hdk::prelude::MockHdkT::new();
        
        debug!("prepare game session element");
        // mock game session element
        let game_params = GameParams {
            regeneration_factor: 1.0,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };
        let game_session = GameSession {
            owner: agent_pubkey_alice.clone(),
            game_params,
            players: players.clone(),
            status: crate::game_session::SessionState::InProgress,       
            scores: PlayerStats::new(), 
        };
        let mut element_with_game_session: Element = fixt!(Element);
        let create_header_1 = fixt!(Create);
        *element_with_game_session.as_header_mut() = Header::Create(create_header_1.clone());
        *element_with_game_session.as_entry_mut() = ElementEntry::Present(game_session.clone().try_into().unwrap());
        let game_session_header_hash_closure = element_with_game_session.header_address().clone();

        // Game round one
        let game_round_zero = GameRound {
            round_num: 0,
            round_state: RoundState {
                resource_amount: 100,
                player_stats: new_player_stats(&players),
            },
            game_moves: vec![],
            session: element_with_game_session.header_address().clone(),
        };

        debug!("prepare game round element");
        let mut element_with_game_round: Element = fixt!(Element);
        let create_header_2 = fixt!(Create);
        let game_round_header_hash_closure = element_with_game_round.header_address().clone();
        *element_with_game_round.as_header_mut() = Header::Create(create_header_2.clone());
        *element_with_game_round.as_entry_mut() = ElementEntry::Present(game_round_zero.clone().try_into().unwrap());
        debug!("game round header hash: {:?}", element_with_game_round.header_address().clone());

        // Game round one - update
        let game_round_updated_to_round_one = GameRound {
            round_num: 1,
            round_state: RoundState{
                player_stats: new_player_stats(&players),
                resource_amount: -10,
            },
            session: element_with_game_session.header_address().clone(),
            game_moves: vec![],
        };

        debug!("prepare game round element");
        let mut element_with_game_round_update: Element = fixt!(Element);
        let update_header_1 = fixt!(Update);
        let game_round_update_header_hash_closure = element_with_game_round_update.header_address().clone();
        *element_with_game_round_update.as_header_mut() = Header::Update(update_header_1.clone());
        *element_with_game_round_update.as_entry_mut() = ElementEntry::Present(game_round_updated_to_round_one.clone().try_into().unwrap());
        debug!("game round header hash: {:?}", element_with_game_round_update.header_address().clone());
       
        // game move alice
        debug!("prepare game move element alice");
        let game_move_alice = GameMove {
            owner: agent_pubkey_alice.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 10,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_alice = fixt!(Element);
        let create_header_3 = fixt!(Create);
        *element_with_game_move_alice.as_header_mut() = Header::Create(create_header_3.clone());
        *element_with_game_move_alice.as_entry_mut() =
            ElementEntry::Present(game_move_alice.try_into().unwrap());
        let move_alice_round1_entry_hash = Header::Create(create_header_3.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_alice_header_hash_closure = element_with_game_move_alice.header_address().clone();


        // game move bob
        debug!("prepare game move element bob");
        let game_move_bob = GameMove {
            owner: agent_pubkey_bob.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 100,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_bob = fixt!(Element);
        let create_header_4 = fixt!(Create);
        *element_with_game_move_bob.as_header_mut() = Header::Create(create_header_4.clone());
        *element_with_game_move_bob.as_entry_mut() =
            ElementEntry::Present(game_move_bob.try_into().unwrap());
        let move_bob_round1_entry_hash = Header::Create(create_header_4.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_bob_header_hash_closure = element_with_game_move_bob.header_address().clone();


        // links
        debug!("prepare game move links");
        let move_alice_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_alice_round1 = Link {
            target: move_alice_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_alice_round1_link_header_hash.into(),
        };

        let move_bob_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_bob_round1 = Link {
            target: move_bob_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_bob_round1_link_header_hash.into(),
        };
        let game_moves: Links = vec![link_to_move_alice_round1, link_to_move_bob_round1].into();


        let header_hash_closure = element_with_game_round.header_address().clone();

        debug!("mock get game round");
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_round.header_address().clone().into(),
            GetOptions::latest(),
            )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_round.clone())]));

        debug!("mock get game session");
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_session.header_address().clone().into(),
            GetOptions::content(),
            )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_session.clone())]));

        debug!("mock get links");
        mock_hdk
            .expect_get_links()
            .times(1)
            .return_once(move |_| Ok(vec![game_moves]));
        
        debug!("mock get game move alice");   
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_alice_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_alice)]));

        debug!("mock get game move bob");   
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_bob_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_bob)]));


        let game_session_update_header_hash = fixt!(HeaderHash);
        let game_session_update_header_hash_closure = game_session_update_header_hash.clone();
        debug!("mock update game session");
        mock_hdk
            .expect_update()
            .times(1)
            .return_once(move |_| Ok(game_session_update_header_hash_closure));
        

        debug!("mock signaling game ended");
        mock_hdk
            .expect_remote_signal()
            .times(1)
            .return_once(move |_| Ok(()));          
        
        hdk::prelude::set_hdk(mock_hdk);
        let result = try_to_close_round(game_round_header_hash_closure.clone());
        assert_eq!(result.unwrap(), game_session_update_header_hash);
    }


    #[test]
    fn test_try_to_close_round_end_game_all_rounds_played(){
        // clear && cargo test --features "mock" --package tragedy_of_commons --lib -- game_round::tests::test_try_to_close_round_end_game_all_rounds_played --exact --nocapture

        enable_tracing(tracing::Level::DEBUG);

        let agent_pubkey_alice = fixt!(AgentPubKey);
        let agent_pubkey_bob = fixt!(AgentPubKey);
        let players = vec![agent_pubkey_alice.clone(), agent_pubkey_bob.clone()];

        let mut mock_hdk = hdk::prelude::MockHdkT::new();
        
        debug!("prepare game session element");
        // mock game session element
        let game_params = GameParams {
            regeneration_factor: 1.0,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };
        let game_session = GameSession {
            owner: agent_pubkey_alice.clone(),
            // status: SessionState::InProgress,
            game_params,
            players: players.clone(),
            status: crate::game_session::SessionState::InProgress,
            scores: PlayerStats::new(),
        };
        let mut element_with_game_session: Element = fixt!(Element);
        let create_header_1 = fixt!(Create);
        *element_with_game_session.as_header_mut() = Header::Create(create_header_1.clone());
        *element_with_game_session.as_entry_mut() = ElementEntry::Present(game_session.clone().try_into().unwrap());
        let game_session_header_hash_closure = element_with_game_session.header_address().clone();

        // Game round three (of 3 rounds specified in game params)
        let game_round_three = GameRound {
            round_num: 3,
            round_state: RoundState{
                resource_amount: 100,
                player_stats: new_player_stats(&players),
            },
            session: element_with_game_session.header_address().clone(),
            game_moves: vec![],
        };

        debug!("prepare game round element");
        let mut element_with_game_round: Element = fixt!(Element);
        let create_header_2 = fixt!(Create);
        let game_round_header_hash_closure = element_with_game_round.header_address().clone();
        *element_with_game_round.as_header_mut() = Header::Create(create_header_2.clone());
        *element_with_game_round.as_entry_mut() = ElementEntry::Present(game_round_three.clone().try_into().unwrap());
        debug!("game round header hash: {:?}", element_with_game_round.header_address().clone());

        // Game round one - update
        let game_round_three_update = GameRound {
            round_num: 3,
            round_state: RoundState{
                resource_amount: 100,
                player_stats: new_player_stats(&players),
            },
            session: element_with_game_session.header_address().clone(),
            game_moves: vec![],
        };


        debug!("prepare game round element");
        let mut element_with_game_round_update: Element = fixt!(Element);
        let update_header_1 = fixt!(Update);
        let game_round_update_header_hash_closure = element_with_game_round_update.header_address().clone();
        *element_with_game_round_update.as_header_mut() = Header::Update(update_header_1.clone());
        *element_with_game_round_update.as_entry_mut() = ElementEntry::Present(game_round_three_update.clone().try_into().unwrap());
        debug!("game round header hash: {:?}", element_with_game_round_update.header_address().clone());

        // Game round two
        // let game_scores = GameScores {
        //     session: HeaderHashB64::from(element_with_game_session.header_address().clone()),
        //     //resources_left: 50,
        //     stats: new_player_stats(vec![agent_pubkey_alice.clone(), agent_pubkey_bob.clone()]), // TODO alter stats
        //     // player_moves: vec![],  // TODO add moves
        // };

        // debug!("prepare game scores element");

        // let mut element_with_game_scores: Element = fixt!(Element);
        // let create_header_game_scores = fixt!(Create);
        // *element_with_game_scores.as_header_mut() = Header::Create(create_header_game_scores.clone());
        // *element_with_game_scores.as_entry_mut() = ElementEntry::Present(game_scores.clone().try_into().unwrap());
        // let game_scores_header_hash_closure = element_with_game_scores.header_address().clone();
        // let game_scores_header_hash_closure2 = element_with_game_scores.header_address().clone();
        // let game_scores_entry_hash_closure = element_with_game_scores.header().entry_hash().clone().unwrap().clone();
        // debug!("game round header hash: {:?}", element_with_game_scores.header_address().clone());
        
        
        // game move alice
        debug!("prepare game move element alice");
        let game_move_alice = GameMove {
            owner: agent_pubkey_alice.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 10,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_alice = fixt!(Element);
        let create_header_3 = fixt!(Create);
        *element_with_game_move_alice.as_header_mut() = Header::Create(create_header_3.clone());
        *element_with_game_move_alice.as_entry_mut() =
            ElementEntry::Present(game_move_alice.try_into().unwrap());
        let move_alice_round1_entry_hash = Header::Create(create_header_3.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_alice_header_hash_closure = element_with_game_move_alice.header_address().clone();


        // game move bob
        debug!("prepare game move element bob");
        let game_move_bob = GameMove {
            owner: agent_pubkey_bob.clone().into(),
            //previous_round: prev_round_entry_hash.clone().into(),
            resources: 10,
            round: element_with_game_round.header_address().clone(),
        };
        let mut element_with_game_move_bob = fixt!(Element);
        let create_header_4 = fixt!(Create);
        *element_with_game_move_bob.as_header_mut() = Header::Create(create_header_4.clone());
        *element_with_game_move_bob.as_entry_mut() =
            ElementEntry::Present(game_move_bob.try_into().unwrap());
        let move_bob_round1_entry_hash = Header::Create(create_header_4.clone()).entry_hash().unwrap().clone();
        let element_with_game_move_bob_header_hash_closure = element_with_game_move_bob.header_address().clone();


        // links
        debug!("prepare game move links");
        let move_alice_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_alice_round1 = Link {
            target: move_alice_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_alice_round1_link_header_hash.into(),
        };

        let move_bob_round1_link_header_hash = HeaderHashB64::from(fixt!(HeaderHash));
        let link_to_move_bob_round1 = Link {
            target: move_bob_round1_entry_hash.clone(),
            timestamp: Timestamp::from(chrono::offset::Utc::now()),
            tag: LinkTag::new("game_move"),
            create_link_hash: move_bob_round1_link_header_hash.into(),
        };
        let game_moves: Links = vec![link_to_move_alice_round1, link_to_move_bob_round1].into();


        let header_hash_closure = element_with_game_round.header_address().clone();

        debug!("mock get game round");
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_round.header_address().clone().into(),
            GetOptions::latest(),
            )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_round.clone())]));

        debug!("mock get game session");
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            element_with_game_session.header_address().clone().into(),
            GetOptions::content(),
            )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_session.clone())]));

        debug!("mock get links");
        mock_hdk
            .expect_get_links()
            .times(1)
            .return_once(move |_| Ok(vec![game_moves]));
        
        debug!("mock get game move alice");   
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_alice_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_alice)]));

        debug!("mock get game move bob");   
        mock_hdk
        .expect_get()
        .with(mockall::predicate::eq(vec![GetInput::new(
            move_bob_round1_entry_hash.clone().into(),
            GetOptions::content(),
        )]))
        .times(1)
        .return_once(move |_| Ok(vec![Some(element_with_game_move_bob)]));

        let game_session_update_header_hash = fixt!(HeaderHash);
        let game_session_update_header_hash_closure = game_session_update_header_hash.clone();
        debug!("mock update game session");
        mock_hdk
            .expect_update()
            .times(1)
            .return_once(move |_| Ok(game_session_update_header_hash_closure));

        debug!("mock signal");
        mock_hdk
            .expect_remote_signal()
            .times(1)
            .return_once(move |_| Ok(()));          
        
        hdk::prelude::set_hdk(mock_hdk);
        let result = try_to_close_round(game_round_header_hash_closure.clone());
        assert_eq!(result.unwrap(), game_session_update_header_hash.clone());
    }


}
