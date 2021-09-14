
use game_move::GameMove;
use game_session::GameParams;
// use game_session::GameSession;
use hdk::prelude::*;
use holo_hash::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use utils::{entry_from_element_create_or_update};

use crate::{utils::entry_hash_from_element};
#[allow(unused_imports)]
use crate::{
    game_move::GameMoveInput,
    game_session::{
        GameSession, GameSessionInput, GameSignal, SessionState, SignalPayload, OWNER_SESSION_TAG,
        PARTICIPANT_SESSION_TAG,
    },
};
mod error;
mod game_move;
mod game_round;
mod game_session;
mod types;
mod utils;


pub fn get_player_profiles_for_game_code(
    short_unique_code: String,
) -> ExternResult<Vec<PlayerProfile>> {

    let anchor = anchor("GAME_CODES".into(), short_unique_code)?;
    debug!("anchor: {:?}", anchor);
    let links: Links = get_links(anchor, Some(LinkTag::new("PLAYER")))?;
    debug!("links: {:?}", links);
    let mut players = vec![];
    for link in links.into_inner() {
        debug!("link: {:?}", link);
        let element: Element = get(link.target, GetOptions::default())?
            .ok_or(WasmError::Guest(String::from("Entry not found")))?;
        let entry_option = element.entry().to_app_option()?;
        let entry: PlayerProfile = entry_option.ok_or(WasmError::Guest(
            "The targeted entry is not agent pubkey".into(),
        ))?;
        players.push(entry);
    }

    Ok(players) // or more Rust like: anchor.into())
}
