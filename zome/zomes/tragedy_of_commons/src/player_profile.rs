use crate::game_code::{create_game_code_anchor, calculate_game_code_anchor_entry_hash};
use hdk::prelude::*;

pub const PLAYER_LINK_TAG: &str = "PLAYER";

/// Actual Holochain entry that stores user's profile
/// for the specific game
#[hdk_entry(id = "player_profile", visibility = "public")]
#[derive(Clone)]
pub struct PlayerProfile {
    pub player_id: AgentPubKey,
    pub nickname: String,
}

/// Struct to receive user input from the UI when user
/// wants to join the game
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct JoinGameInfo {
    pub gamecode: String,
    pub nickname: String,
}

/// Creates a PlayerProfile instance, commits it as a Holochain entry
/// and returns a hash value of this entry
pub fn create_and_hash_entry_player_profile(nickname: String) -> ExternResult<EntryHash> {
    let agent = agent_info()?;
    debug!(
        "create_and_hash_entry_player_profile | nickname: {}, agent {:?}",
        nickname,
        agent.clone()
    );
    let player_profile = PlayerProfile {
        player_id: agent.agent_initial_pubkey, // bad design for real apps 1/ initial_pubkey is linked to app itself, so no roaming profile 2/ lost if app is reinstalled (= basicly new user)
        nickname,
    };
    create_entry(&player_profile)?;
    debug!("create_and_hash_entry_player_profile | profile created, hashing");
    hash_entry(&player_profile)
}

/// Creates user's profile for the game and registers this user as one of the game players
/* 
When you create an anchor with the function return a EntryHash. Once you
know the entry_hash of an anchor it is best to use the get_anchor(entry_hash) fn to retrieve
this anchor, when you need it. In the case of the devcamp game, we have a little problem.
Players share the game_code via chat or voice or video... That means that the player who
initiated the game, the game leader, knows the entry_hash of the game code, but players that
want to join the game do not. Other players need to be able to find the same anchor if they
want to join the game. Of course the game leader could communicate the entry hash, but that
is not as convenient as passing the much shorter game code.
So for other players that do not have the game code the problem exists in finding out the
entry hash of the anchor while they only have game code.

There are 2 approaches you can take to solve this problem, each with it own benefits.
1/ Other players can take the game_code and calculate the hash, without actually creating
    a anchor in the DHT (with the same entry hash, but a different header hash). Like we do
    in player_profile::get_game_code_anchor
Benefits: less DHT operations, no extra header in the DHT
Downside: calculating the entry_hash and fetching the anchor with this hash via 'get_anchor',
            does not guarantee that anchor will be found at the point in time that you start
            searching it. Even if you have a entry_hash of entry that absolutely, 100% exists.
            It does not guarantee it can be found in your part of the DHT, yet. Eventually it
            will be.The downside is you to need poll until you find the anchor. This how you
            could calculate a entry hash:
    let path: Path = (&Anchor {
            anchor_type: GAME_CODES_ANCHOR.into(),
            anchor_text: Some(game_code),
        })
        .into();
    let anchor_hash = path.hash()
2/ The other way is for the other players to create the same anchor. Which we do here by calling
player_profile::create_game_code_anchor. The anchor entry will be
created again. It will add a header and a entry to the DHT. But since the entry has the same
entry_hash it will already be stored.
Benefit: entry is added to your source chain before being sent to the DHT, so it is
immediately available. No polling needed
Downside: More DHT ops, extra header in the DHT
*/
pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHash> {
    info!("join_game_with_code | input: {:#?}", input);
    let anchor = create_game_code_anchor(input.gamecode)?;
    debug!("join_game_with_code | anchor created {:#?}", &anchor);
    let player_profile_entry_hash = create_and_hash_entry_player_profile(input.nickname)?;
    debug!(
        "join_game_with_code | profile entry hash {:?}",
        &player_profile_entry_hash
    );
    create_link(
        anchor.clone().into(),
        player_profile_entry_hash.into(),
        LinkTag::new(String::from(PLAYER_LINK_TAG)),
    )?;
    debug!("join_game_with_code | link created");
    Ok(anchor) // or more Rust like: anchor.into())
}

pub fn get_player_profiles_for_game_code(
    short_unique_code: String,
) -> ExternResult<Vec<PlayerProfile>> {
    let anchor = calculate_game_code_anchor_entry_hash(short_unique_code)?;
    debug!("anchor: {:?}", anchor);
    let links: Links = get_links(anchor, Some(LinkTag::new(String::from(PLAYER_LINK_TAG))))?;
    debug!("links: {:#?}", links);
    let mut players = vec![];
    for link in links.into_inner() {
        debug!("link: {:#?}", link);
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

pub fn get_players_for_game_code(short_unique_code: String) -> ExternResult<Vec<PlayerProfile>> {
    // Ok(vec!["Anipur".into(), "Bob".into()]);

    debug!("get profiles");
    let player_profiles = get_player_profiles_for_game_code(short_unique_code)?;

    // debug!("filter profiles to extract nickname");
    let players: Vec<String> = player_profiles.iter().map(|x| x.nickname.clone()).collect();
    debug!("players: {:#?}", players);
    debug!("profiles {:#?}", player_profiles);
    Ok(player_profiles) // or more Rust like: anchor.into())
}
