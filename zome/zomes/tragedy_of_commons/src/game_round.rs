use hdk::prelude::*;
use holo_hash::{EntryHashB64, HeaderHashB64};
use std::collections::HashMap;
use crate::types::{ReputationAmount,ResourceAmount};

// todo: rename it so we don't have name clash with SessionState
#[derive(Clone, Debug, Serialize, Deserialize)]
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

// NOTE: game round is always created once players made their moves, so every round is always
// a retrospective of moves made, not created before and updated later

// todo: implement round lost logic in round methods

fn calculate_round_state(params: GameParams, player_moves: Vec<?>) -> () {
    // todo:
    // calculate game state from the player moves
    // 
}

fn new_game_round(session: EntryHash, latest_round: Option<HeaderHashB64>, player_moves: Vec<?>) -> ExternResult<EntryHashB64> {
    // let state = calculate_round_state
    // if latest_round not None:
    //  update existing round entry on the latest_round hash
    // else:
    //  create new round entry
    //  make a link from session -> round
    // if round is finished or lost: 
    //  update game session state
    
}

