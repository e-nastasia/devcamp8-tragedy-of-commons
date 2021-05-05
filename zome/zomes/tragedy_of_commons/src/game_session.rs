use crate::game_round::GameRound;
use crate::types::ResourceAmount;
use hdk::prelude::*;
use holo_hash::EntryHashB64;
use holo_hash::HeaderHashB64;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    InProgress,
    Lost { last_round: EntryHash },
    // TODO: when validating things, check that last game round is finished to verify
    // that session itself is finished
    Finished { last_round: EntryHash },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameParams {
    pub regeneration_factor: f32,
    pub start_amount: ResourceAmount,
    pub num_rounds: u32,
    pub resource_coef: u32,
    pub reputation_coef: u32,
}

#[hdk_entry(id = "game_session", visibility = "public")]
pub struct GameSession {
    pub owner: AgentPubKey,
    // pub created_at: Timestamp,
    pub status: SessionState,
    pub game_params: GameParams,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSessionInput {
    pub game_params: GameParams,
    pub players: Vec<AgentPubKey>,
}

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


//external function that can be called from UI/test, until invitation zoom is added.
#[hdk_extern]
pub fn start_new_session(player_list:Vec<AgentPubKey>) -> ExternResult<HeaderHashB64> {
    let input = GameSessionInput {
    game_params: GameParams {
        regeneration_factor: 1.1,
        start_amount: 100,
        num_rounds: 3,
        resource_coef: 3,
        reputation_coef: 2,
        },
    players: player_list,
    };
    new_session(input)

}

pub fn new_session(input: GameSessionInput) -> ExternResult<HeaderHashB64> {
    // NOTE: we create a new session already having invites answered by everyone invited
    // and invite zome handles invite process before this fn call
    let agent_info: AgentInfo = agent_info()?; // agent that starts new game

    // todo:
    // get timestamp

    let latest_pubkey = agent_info.agent_latest_pubkey;
    // create entry for game session
    let gs = GameSession {
        owner: latest_pubkey.clone(),
        status: SessionState::InProgress,
        game_params: input.game_params,
    };
    let header_hash = create_entry(&gs)?;
    let entry_hash_game_session = hash_entry(&gs)?;

    // make link from every player to agent address to game session entry
    create_link(latest_pubkey.clone().into(), entry_hash_game_session, LinkTag::new("my_game_sessions"))?;

    // use remote signals from RSM to send a real-time notif to invited players
    //  ! using remote signal to ping other holochain backends, instead of emit_signal
    //  that would talk with the UI
    // NOTE: we're sending signals to notify that players need to make their moves
    // TODO: include current round number, 0 , in notif data  

    let payload = ExternIO::encode(SignalPayload::StartRound(gs))?;
    remote_signal(
        payload,
        input.players.clone(),
    )?;
    tracing::debug!("sending signal to {:?}", input.players.clone());

    // // todo: get timestamp as systime
    // create_entry(&calendar_event)?;

    Ok(HeaderHashB64::from(header_hash))
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
#[serde(tag = "signal_name", content = "signal_payload")]
pub enum SignalPayload {
    StartRound(GameSession),
    GameStopped,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    use std::vec;

    #[test]
    fn test_new_session() {
        let mut mock_hdk = hdk::prelude::MockHdkT::new();
        let game_params = GameParams {
            regeneration_factor: 1.1,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };

        // mock agent info
        let agent_pubkey = fixt!(AgentPubKey);
        let agent_info = AgentInfo::new(agent_pubkey.clone(), agent_pubkey.clone());

        mock_hdk
            .expect_agent_info()
            .times(1)
            .return_once(move |_| Ok(agent_info));

        // mock create entry
        let headerhash = fixt!(HeaderHash);

        let entryhash = fixt!(EntryHash);
        let closure_header_hash = headerhash.clone();
        mock_hdk
        .expect_create()
        .with(hdk::prelude::mockall::predicate::eq(
            EntryWithDefId::try_from(GameSession {
                owner: agent_pubkey.clone(),
                status: SessionState::InProgress,
                game_params: game_params.clone(),
            })
                .unwrap(),
            ))
            .times(1)
            .return_once(move |_| Ok(closure_header_hash));
            
            let input = GameSessionInput {
                game_params: game_params,
                players: vec![fixt!(AgentPubKey), fixt!(AgentPubKey), fixt!(AgentPubKey)], // 3 random players
            };
            
            let entry_hash_game_session = fixt!(EntryHash);
            mock_hdk
            .expect_hash_entry()
            .times(1)
            .return_once(move |_| Ok(entry_hash_game_session));

            mock_hdk
            .expect_remote_signal()
            .times(1)
            .return_once(move |_| Ok(()));
            
            let header_hash_link = fixt!(HeaderHash);
            mock_hdk
            .expect_create_link()
            .times(1)
            .return_once(move |_| Ok(header_hash_link));
            
            hdk::prelude::set_hdk(mock_hdk);
            new_session(input);
        }
    }
