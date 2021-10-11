use std::collections::BTreeMap;
use crate::game_move::GameMove;
use hdk::prelude::*;

pub type ResourceAmount = i32;
pub type PlayerStats = BTreeMap<AgentPubKey, ResourceAmount>;

/// Generates empty PlayerStats with 0 values for every player in players
pub fn new_player_stats(players: &Vec<AgentPubKey>) -> PlayerStats {
    players
        .into_iter()
        .map(|pub_key| (pub_key.clone(), 0))
        .collect::<PlayerStats>()
}

/// Generates PlayerStats instance with the state from the input game_moves
pub fn player_stats_from_moves(game_moves: Vec<GameMove>) -> PlayerStats {
    game_moves
        .into_iter()
        .map(|m| (m.owner.clone(), m.resources))
        .collect::<PlayerStats>()
}
