use hdk::prelude::*;

// Since we'll be using a hardcoded string value to access all game code,
// we'd better declare it as a constant to be re-used
// Note: we're using &str instead of String type here because size of this string
// is known at compile time, so there's no need to allocate memory dynamically
// by using String.
// More about &str and String difference here:
// https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#the-string-type
pub const GAME_CODES_ANCHOR: &str = "GAME_CODES";

/// Creates anchor for a new game identified by the short_unique_code
/// and registers it under GAME_CODES_ANCHOR to be discoverable
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), short_unique_code)?;
    // Note the lack of ; in the end of the next code line: this is the value we return here
    // More on that syntax here:
    // https://doc.rust-lang.org/stable/book/ch03-03-how-functions-work.html#functions-with-return-values
    Ok(anchor)
}

/// Retrieves entry hash of the game code anchor that corresponds
/// to the game_code provided
pub fn get_game_code_anchor(game_code: String) -> ExternResult<EntryHash> {
    anchor(GAME_CODES_ANCHOR.into(), game_code.clone())
}
