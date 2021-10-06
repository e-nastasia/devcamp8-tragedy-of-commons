use hdk::prelude::*;

// We apply macro to the the line of code that follows, allowing to compile code
// from the game_code module even if some of this code is unused.
// (Default compiler's behavior is to treat unused code as errors)
// We'll remove this line later once we start using all code from the game_code module.
// "unused" here is name of the lint group, and there are actually a lot of those!
// Check out this link for more details:
// https://doc.rust-lang.org/rustc/lints/groups.html
#[allow(unused)]
mod game_code;
#[allow(unused)]
mod player_profile;

// This is another macro applied to the function that follows, and we need it to
// expose this function as part of our backend API
#[hdk_extern]
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    game_code::create_game_code_anchor(short_unique_code)
}