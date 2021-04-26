use hdk::prelude::*;
use holo_hash::EntryHashB64;
use crate::types::ResourceAmount;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionState {
    InProgress,
    Lost { last_round: EntryHash },
    // TODO: when validating things, check that last game round is finished to verify
    // that session itself is finished
    Finished { last_round: EntryHash }
}


#[hdk_entry(id = "game_session", visibility = "public")]
pub struct GameSession {
    pub owner: AgentPubKey,
    pub regeneration_factor: f32,
    pub starting_amount_of_resources: ResourceAmount,
    pub resource_coef: u32,
    pub reputation_coef: u32,
    pub created_at: Timestamp,
    pub invited: Vec<AgentPubKey>,
    pub status: SessionState,
    pub num_rounds: u32,
}

// TODO: separate game input params into another struct to include in both
// GameSEssion and SessionInput
pub struct GameSessionInput {
    regeneration_factor: f32,
    start_amount: ResourceAmount,
    resource_coef: u32,
    reputation_coef: u32,
    num_rounds: u32,
    invited: Vec<AgentPubKey>,
}

pub struct GameParams {
    regeneration_factor: f32,
    start_amount: ResourceAmount,
    num_rounds: u32,
}


impl GameSession {
    
    // called in different contexts:
    // if validation: if round isn't available, validation sin't finished
    // if session state update: round is available
    pub fn update_state(&self, game_round: GameRound) -> () {
        // this is called every time after GameRound is created
        

        // if round is lost <= 0:
        //  game session is lost
        // elif number round == num_rounds:
        //  game session is finished
        // else:
        //  game session is in progress
        
    }
    
    
    
    
}

pub fn new_session(input: GameSessionInput) -> ExternResult<EntryHashB64> {
    // NOTE: we create a new session already having invites answered by everyone invited
    // and invite zome handles invite process before this fn call
    let agent_info = agent_info()?;

    // todo:
    // get timestamp
    // create entry
    // make link from agent address to game session entry
    // use remote signals from RSM to send a real-time notif to invited players
    //  ! using remote signal to ping other holochain backends, instead of emit_signal
    //  that would talk with the UI
    // NOTE: we're sending signals to notify that players need to make their moves


    // let new_session = GameSession {
    //     owner: agent_info,
    //     regeneration_factor
    // }
    
    // // todo: get timestamp as systime
    // create_entry(&calendar_event)?;
}


