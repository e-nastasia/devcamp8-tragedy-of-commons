

use hdk::prelude::*;
use holo_hash::{EntryHashB64, HeaderHashB64};
use std::collections::HashMap;
use crate::types::{ReputationAmount,ResourceAmount};
use crate::game_session::GameParams;
use crate::game_move::GameMove;



// todo: rename it so we don't have name clash with SessionState
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundState {
    pub resource_amount: ResourceAmount,
    pub player_stats: HashMap<AgentPubKey, (ResourceAmount, ReputationAmount)>,
}

#[hdk_entry(id = "game_round", visibility = "public")]
pub struct GameRound {
    pub round_num: u32,
    pub session: EntryHash,
    pub round_state: RoundState,
    pub previous_round_moves: Vec<EntryHash>
}

// NOTE: this fn would be used both in validation and when creating game round entries
// so it has to be very lightweight and can not make any DHT queries
fn calculate_round_state(params: GameParams, player_moves: Vec<GameMove>) -> RoundState {
    // todo:
    // calculate round state from the player moves
    // 
    let consumed_resources_in_round:u32 = player_moves.iter().map(|x| x.resources).sum();
    params.start_amount - consumed_resources_in_round
    // unimplemented!()
}



// NOTE: game round is always created once players made their moves, so every round is always
// a retrospective of moves made, not created before and updated later
// NOTE: given the retrospective nature, maybe we should call this fn "close current round" or
// "start next round" to avoid adding more confusion
fn new_game_round(session: EntryHash, previous_round: Option<HeaderHashB64>, player_moves: Vec<GameMove>) -> ExternResult<EntryHashB64> {
    // validate that player_moves.len() == session.game_params.invited.len(),
    // otherwise current round isn't complete and we can't create a new one
    
    // let state = calculate_round_state
    // if latest_round not None:
    //  update existing round entry on the latest_round hash (continuing the update chain)
    // else:
    //  create new round entry
    //  make a link from session -> round
    // if round is finished or lost: 
    //  update game session state

    

    unimplemented!()
}




#[cfg(test)]
mod tests {
    use std::vec;
    use super::*;
    use hdk::prelude::*;
    use ::fixt::prelude::*;

    // #[test]
    // fn create_entry_mocked() {
    //     let mut mock_hdk = hdk::prelude::MockHdkT::new();


    #[test]
    fn test_calculate_round_state() {
        let gp = GameParams {
            regeneration_factor: 1.1,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };
        let move1 = GameMove {
            owner: fixt!(AgentPubKey),
            previous_round: Some(fixt!(EntryHash)),
            resources: 5,            
        };
        let move2 = GameMove {
            owner: fixt!(AgentPubKey),
            previous_round: Some(fixt!(EntryHash)),
            resources: 10,            
        };
        let s = calculate_round_state(gp.clone(), vec![move1, move2]);
        assert_eq!(gp.clone().start_amount - 15, s);
    }


}
