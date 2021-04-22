use hdk::prelude::*;
use std::collections::HashMap;

pub type ResourceAmount = u32;
pub type ReputationAmount = i32;

pub struct GameState {
    pub resource_amount: u32,
    pub player_stats: HashMap<AgentPubKey, (ResourceAmount, ReputationAmount)>,
}

#[hdk_entry(id = "game_round", visibility = "public")]
pub struct GameRound {
    pub round_num: u32,
    pub session: EntryHash,
    pub game_state: GameState,
    pub previous_round_moves: Vec<EntryHash>
}