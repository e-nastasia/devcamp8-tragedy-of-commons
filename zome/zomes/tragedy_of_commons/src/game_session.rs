use hdk::prelude::*;
//use holo_hash::EntryHashB64;
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
    start_amount: u32,
    resource_coef: u32,
    reputation_coef: u32,
    num_rounds: u32,
    invited: Vec<AgentPubKey>,
}

/* 
pub fn new_session(input: GameSessionInput) -> ExternResult<EntryHashB64> {
    let agent_info = agent_info()?;
    
    // todo: get timestamp as systime
}
*/