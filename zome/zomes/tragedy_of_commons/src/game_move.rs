use hdk::prelude::*;
use holo_hash::EntryHashB64;
use crate::types::ResourceAmount;

#[hdk_entry(id = "game_move", visibility = "public")]
pub struct GameMove {
    pub owner: AgentPubKey,
    // PROBLEM: if we're creating the GameRound retrospectively, we can't reference it
    // maybe add a link from GameRound -> GameMove?
    // pub round: EntryHash,
    // NOTE: this is similar to what we have in the new_game_round fn: every move refenreces
    // only the previous round, and for the very first one this would be None (since rounds
    // are created retrospectively). Having this field would make sure game moves made by the same
    // player with the same resources value are still treated as separate entries
    pub previous_round: Option<EntryHashB64>,
    pub resources: ResourceAmount,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMoveInput {
    pub resource_amount: ResourceAmount,
    pub previous_round: Option<EntryHashB64>,
}

fn new_move() -> (input: GameMoveInput) {
    // todo: calculate agent address
    // todo: create a GameMove entry
    // todo: call make_new_round to attempt to make a new round
    // 
}
