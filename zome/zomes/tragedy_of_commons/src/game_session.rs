use crate::error::Error;
use crate::types::{new_player_stats, PlayerStats, ResourceAmount};
use crate::utils::{entry_from_element_create_or_update, entry_hash_from_element};
use crate::PlayerProfile;
use crate::{
    game_round::{GameRound, RoundState},
    types::ReputationAmount,
    utils::convert_keys_from_b64,
};

use hdk::prelude::holo_hash::HeaderHashB64;
use hdk::prelude::*;
use holo_hash::{AgentPubKeyB64, EntryHashB64};
use std::{collections::HashMap, time::SystemTime};

pub const OWNER_SESSION_TAG: &str = "my_game_sessions";
pub const PARTICIPANT_SESSION_TAG: &str = "game_sessions";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    InProgress,
    //Lost { last_round: HeaderHash },
    // TODO: when validating things, check that last game round is finished to verify
    // that session itself is finished
    Finished, //{ last_round: HeaderHash },
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
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameSessionInput {
    pub game_params: GameParams,
    pub players: Vec<AgentPubKeyB64>,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SignalPayload {
    pub game_session_header_hash: HeaderHashB64,
    pub round_header_hash_update: HeaderHashB64,
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

/// Create a new GameSession with the confirmed players (who accepted their invites).
/// NOTE: we're only creating session for those who accepted and only if there are at
/// least two of them -- otherwise there won't be any turns.
pub fn new_session(players: Vec<AgentPubKey>, game_params: GameParams) -> ExternResult<HeaderHash> {
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

    // create links from all game players' addresses to the game session entry
    info!("linking participants to game session");
    for p in players.iter() {
        let agent_pub_key_player = p.clone();
        let agent_pub_key_owner = agent_info_owner.agent_initial_pubkey.clone();
        // skip the game creator
        if agent_pub_key_player != agent_pub_key_owner
            && agent_pub_key_player != agent_pub_key_owner
        {
            debug!("================= Creating link from PARTICIPANT address {:?} to game session {:?}", agent_pub_key_player.clone(), game_session_entry_hash.clone());
            create_link(
                agent_pub_key_player.into(),
                game_session_entry_hash.clone(),
                LinkTag::new(PARTICIPANT_SESSION_TAG),
            )?;
        }
    }

    // create game round results for round 0
    // this is starting point for all the game moves of round 1 to reference (implicit link)
    let no_moves: Vec<EntryHash> = vec![];

    // TODO: create a link from session to game round entry to make the round discoverable
    let round_zero = GameRound::new(
        0,
        game_session_header_hash.clone(),
        RoundState::new(
            game_session.game_params.start_amount,
            new_player_stats(&players),
        ),
        no_moves,
    );
    let header_hash_round_zero = create_entry(&round_zero)?;
    let entry_hash_round_zero = hash_entry(&round_zero)?;

    // use remote signals from RSM to send a real-time notif to invited players
    //  ! using remote signal to ping other holochain backends, instead of emit_signal
    //  that would talk with the UI
    // NOTE: we're sending signals to notify that a new round has started and
    // that players need to make their moves
    // WARNING: remote_signal is fire and forget, no error if it fails, might be a weak point if this were production happ
    let signal_payload = SignalPayload {
        game_session_header_hash: game_session_header_hash.clone().into(),
        round_header_hash_update: header_hash_round_zero.clone().into(),
    };
    let signal = ExternIO::encode(GameSignal::StartNextRound(signal_payload))?;
    // Since we're storing agent keys as AgentPubKeyB64, and remote_signal only accepts
    // the AgentPubKey type, we need to convert our keys to the expected data type
    remote_signal(signal, players.clone())?;
    debug!("sending signal to {:?}", players);

    Ok(header_hash_round_zero)
}

pub fn get_sessions_with_tags(
    link_tags: Vec<&str>,
) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
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
            Ok((EntryHashB64::from(link.target.clone()), result))
        })
        .collect::<ExternResult<Vec<(EntryHashB64, GameSession)>>>()?;

    Ok(results)
}

pub fn get_my_own_sessions_via_source_query() -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    let filter = ChainQueryFilter::new()
        .include_entries(true)
        .entry_type(EntryType::App(AppEntryType::new(
            entry_def_index!(GameSession)?,
            zome_info()?.zome_id,
            EntryVisibility::Public,
        )));

    let list_of_elements = query(filter)?;
    let mut list_of_tuples: Vec<(EntryHashB64, GameSession)> = vec![];
    for el in list_of_elements {
        let gs: GameSession = entry_from_element_create_or_update(&el)?;
        let gs_entry_hash: EntryHash = entry_hash_from_element(&el)?.to_owned();
        list_of_tuples.push((EntryHashB64::from(gs_entry_hash), gs));
    }
    Ok(list_of_tuples)
}

pub fn get_sessions_with_status(
    target_state: SessionState,
) -> ExternResult<Vec<(EntryHashB64, GameSession)>> {
    let all_sessions = get_sessions_with_tags(vec![OWNER_SESSION_TAG, PARTICIPANT_SESSION_TAG])?;

    let results = all_sessions
        .into_iter()
        .filter(|entry| entry.1.status == target_state)
        .collect::<Vec<(EntryHashB64, GameSession)>>();

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

#[cfg(test)]
#[rustfmt::skip]   // skipping formatting is needed, because to correctly import fixt we needed "use ::fixt::prelude::*;" which rustfmt does not like
mod tests {
    use super::*;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    use std::vec;
    use super::*;
    use ::mockall::predicate::*;
    use ::mockall::mock;

    #[test]
    fn test_new_session() {
        let game_params = GameParams {
            regeneration_factor: 1.1,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };

        // mock agent info
        let agent_pubkey_owner = fixt!(AgentPubKey);
        let agent_info = AgentInfo::new(agent_pubkey_owner.clone(), agent_pubkey_owner.clone());
        let agent2_pubkey = fixt!(AgentPubKey);
        let players = vec![agent_pubkey_owner.clone(), agent2_pubkey.clone()];

        // mock create entry
        let game_session_header_hash = fixt!(HeaderHash);
        let game_session_entry_hash = fixt!(EntryHash);
        let game_session_header_hash_closure = game_session_header_hash.clone();
        let game_session = GameSession {
            owner: agent_pubkey_owner.clone(),
            status: SessionState::InProgress,
            game_params: game_params.clone(),
            players: players.clone(),
            scores: PlayerStats::new(),
        };

        // round zero
        let round_zero = GameRound {
            game_moves: vec![],
            round_num: 0,
            session: game_session_header_hash.clone(),
            round_state: RoundState{
                player_stats: new_player_stats(&players),
                resource_amount: 100,
            },
        };
        let round_zero_header_hash = fixt!(HeaderHash);
        let round_zero_entry_hash = fixt!(EntryHash);
        let round_zero_header_hash_closure = round_zero_header_hash.clone();


        let mut mock_hdk = hdk::prelude::MockHdkT::new();

        mock_hdk
        .expect_agent_info()
        .times(1)
        .return_once(move |_| Ok(agent_info));

        mock_hdk
            .expect_create()
            .with(mockall::predicate::eq(
                EntryWithDefId::try_from(&game_session)
                .unwrap(),
            ))
            .times(1)
            .return_once(move |_| Ok(game_session_header_hash_closure));

        mock_hdk
            .expect_hash_entry()
            .times(1)
            .return_once(move |_| Ok(game_session_entry_hash));

        // link to owner + link to participant
        mock_hdk
            .expect_create_link()
            .times(1)
            .return_once(move |_| Ok(fixt!(HeaderHash)));
        mock_hdk
            .expect_create_link()
            .times(1)
            .return_once(move |_| Ok(fixt!(HeaderHash)));
        
        mock_hdk
            .expect_create()
            // .with(mockall::predicate::eq(
            //     EntryWithDefId::try_from(&round_zero)
            //     .unwrap(),
            // ))
            .times(1)
            .return_once(move |_| Ok(round_zero_header_hash_closure));

        let entry_hash_game_session = fixt!(EntryHash);
        mock_hdk
            .expect_hash_entry()
            .times(1)
            .return_once(move |_| Ok(round_zero_entry_hash));
        mock_hdk
            .expect_remote_signal()
            .times(1)
            .return_once(move |_| Ok(()));

        let header_hash_link = fixt!(HeaderHash);


        hdk::prelude::set_hdk(mock_hdk);
        
        let result = new_session(players, game_params).unwrap();
        
        assert_eq!(result, round_zero_header_hash);
    }
}
