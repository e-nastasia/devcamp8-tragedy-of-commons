

use hdk::prelude::*;
use holo_hash::{EntryHashB64, HeaderHashB64};
use std::collections::HashMap;
use crate::types::{ReputationAmount,ResourceAmount};
use crate::game_session::GameParams;
use crate::game_move::GameMove;

const no_reputation:ReputationAmount = 0;

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
    
    // resources 
    let consumed_resources_in_round:u32 = player_moves.iter().map(|x| x.resources).sum();
    let total_leftover_resource = params.start_amount - consumed_resources_in_round;

    // player stats
    let mut stats:HashMap<AgentPubKey, (ResourceAmount, ReputationAmount)> = HashMap::new();
    for p in player_moves.iter() {
        let a = p.owner.clone();
        let tuple: (ResourceAmount, ReputationAmount) = (p.resources, no_reputation);
        stats.insert(a, tuple);
    }

    RoundState{
        resource_amount: total_leftover_resource,
        player_stats: stats,
    }

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
    use mockall::mock;

    #[test]
    fn test_calculate_round_state() {

        let gp = GameParams {
            regeneration_factor: 1.1,
            start_amount: 100,
            num_rounds: 3,
            resource_coef: 3,
            reputation_coef: 2,
        };

        let p1_key = fixt!(AgentPubKey);
        let move1 = GameMove {
            owner: p1_key.clone(),
            previous_round: Some(EntryHashB64::from(fixt!(EntryHash))),
            resources: 5,            
        };

        let p2_key = fixt!(AgentPubKey);
        let move2 = GameMove {
            owner: p2_key.clone(),
            previous_round: Some(EntryHashB64::from(fixt!(EntryHash))),
            resources: 10,            
        };
        let s = calculate_round_state(gp.clone(), vec![move1, move2]);
        assert_eq!(gp.clone().start_amount - 15, s.resource_amount);
        
        let stats_p1: (ResourceAmount, ReputationAmount) = *s.player_stats.get(&p1_key.clone()).unwrap();
        assert_eq!(stats_p1.0, 5);
        assert_eq!(stats_p1.1, 0);
        
        let stats_p2: (ResourceAmount, ReputationAmount) = *s.player_stats.get(&p2_key.clone()).unwrap();
        assert_eq!(stats_p2.0, 10);
        assert_eq!(stats_p1.1, 0);

    }


}
