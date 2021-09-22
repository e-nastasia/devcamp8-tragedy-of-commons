use crate::game_code::GAME_CODES_ANCHOR;
use hdk::prelude::holo_hash::{EntryHash, EntryHashB64};
use hdk::prelude::*;

#[hdk_entry(id = "player_profile", visibility = "public")]
#[derive(Clone)]
pub struct PlayerProfile {
    pub player_id: AgentPubKey,
    pub nickname: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct JoinGameInfo {
    pub gamecode: String,
    pub nickname: String,
}

pub fn create_and_hash_entry_player_profile(nickname: String) -> ExternResult<EntryHash> {
    let agent = agent_info()?;
    debug!(
        "create_and_hash_entry_player_profile | nickname: {}, agent {:?}",
        nickname,
        agent.clone()
    );
    let player_profile = PlayerProfile {
        player_id: agent.agent_initial_pubkey, // bad design for real apps 1/ initial_pubkey is linked to app itself, so no roaming profile 2/ lost if app is reinstalled (= basicly new user)
        nickname: nickname,
    };
    create_entry(&player_profile)?;
    debug!("create_and_hash_entry_player_profile | profile created, hashing");
    hash_entry(&player_profile)
}

/// Creates user's profile and registers this user as one of the game players
pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHashB64> {
    info!(
        "join_game_with_code | input: {:?}, game code: {:?}",
        input, input.gamecode
    );
    let anchor = anchor(GAME_CODES_ANCHOR.into(), input.gamecode)?;
    debug!("join_game_with_code | anchor created {:?}", &anchor);
    let player_profile_entry_hash = create_and_hash_entry_player_profile(input.nickname)?;
    debug!(
        "join_game_with_code | profile entry hash {:?}",
        &player_profile_entry_hash
    );
    create_link(
        anchor.clone().into(),
        player_profile_entry_hash.into(),
        LinkTag::new("PLAYER"),
    )?;
    debug!("join_game_with_code | link created");
    Ok(EntryHashB64::from(anchor)) // or more Rust like: anchor.into())
}

pub fn get_player_profiles_for_game_code(
    short_unique_code: String,
) -> ExternResult<Vec<PlayerProfile>> {
    let anchor = anchor(GAME_CODES_ANCHOR.into(), short_unique_code)?;
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

pub fn get_players_for_game_code(short_unique_code: String) -> ExternResult<Vec<PlayerProfile>> {
    // Ok(vec!["Anipur".into(), "Bob".into()]);

    debug!("get profiles");
    let player_profiles = get_player_profiles_for_game_code(short_unique_code)?;

    // debug!("filter profiles to extract nickname");
    let players: Vec<String> = player_profiles.iter().map(|x| x.nickname.clone()).collect();
    debug!("players: {:?}", players);
    debug!("profiles {:?}", player_profiles);
    Ok(player_profiles) // or more Rust like: anchor.into())
}
