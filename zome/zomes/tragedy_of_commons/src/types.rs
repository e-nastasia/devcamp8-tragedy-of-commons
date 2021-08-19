use std::collections::HashMap;
// NOTE(e-nastasia): I don't like that we include everything here, I'd like to make
// that import more precise. But maybe that's ok?
use hdk::prelude::*;

pub type ResourceAmount = i32;
pub type ReputationAmount = i32;
// TODO(e-nastasia): how do we make this datatype serializable?
pub type PlayerStats = HashMap<AgentPubKey, (ResourceAmount, ReputationAmount)>;

/// Generates empty PlayerStats with 0 values for every player in players
pub fn new_player_stats(players: &Vec<AgentPubKey>) -> PlayerStats {
    players
        .into_iter()
        .map(|pub_key| (pub_key.clone(), (0, 0)))
        .collect::<PlayerStats>()
}
