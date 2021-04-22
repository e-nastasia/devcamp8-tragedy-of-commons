use hdk::prelude::*;
use std::collections::HashMap;
use crate::game_round::ResourceAmount;

#[hdk_entry(id = "game_move", visibility = "public")]
pub struct GameMove {
    pub owner: AgentPubKey,
    pub round: EntryHash,
    pub resources: ResourceAmount,
}