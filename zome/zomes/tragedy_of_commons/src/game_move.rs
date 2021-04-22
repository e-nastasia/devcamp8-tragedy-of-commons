use hdk::prelude::*;
use crate::types::ResourceAmount;

#[hdk_entry(id = "game_move", visibility = "public")]
pub struct GameMove {
    pub owner: AgentPubKey,
    pub round: EntryHash,
    pub resources: ResourceAmount,
}