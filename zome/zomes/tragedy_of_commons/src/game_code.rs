use hdk::prelude::*;

pub const GAME_CODES_ANCHOR: &str = "GAME_CODES";

/// Creates anchor for a new game identified by the short_unique_code
/// and registers it under GAME_CODES_ANCHOR to be discoverable
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), short_unique_code)?;
    Ok(anchor) // or more Rust like: anchor.into())
}

/// Retrieves entry hash of the game code anchor that corresponds
/// to the game_code provided
pub fn get_game_code_anchor(game_code: String) -> ExternResult<EntryHash> {
    /* Since do not know the hash of the anchor, because only the game code is known,
    we have to calculate the hash.
    */
    let path: Path = (&Anchor {
        anchor_type: GAME_CODES_ANCHOR.into(),
        anchor_text: Some(game_code),
    })
        .into();
    path.hash()
}
