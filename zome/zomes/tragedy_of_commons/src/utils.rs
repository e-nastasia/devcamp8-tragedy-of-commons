use hdk::prelude::*;
use holo_hash::AgentPubKeyB64;

use crate::error::Error;
// NOTE: didn't had time to figure out how to apply this once on a lib level
// TODO: remove it later
#[allow(dead_code)]

pub fn try_get_and_convert<T: TryFrom<Entry>>(entry_hash: EntryHash) -> ExternResult<T> {
    match get(entry_hash.clone(), GetOptions::default())? {
        Some(element) => try_from_element(element),
        None => Err(crate::err("Entry not found")),
    }
}

pub fn try_from_element<T: TryFrom<Entry>>(element: Element) -> ExternResult<T> {
    match element.entry() {
        element::ElementEntry::Present(entry) => {
            T::try_from(entry.clone()).or(Err(crate::err("Cannot conver entry")))
        }
        _ => Err(crate::err("Could not convert element")),
    }
}

/// Converts binary string pub keys into binary array pub keys.
/// Binary string format is used for sending data to UI,
/// and binary array format is used for working with keys on the backend
/// TODO(e-nastasia): I think it may make sense to keep agent pub keys as binary arrays
/// and only convert to binary string when sending data to UI?
pub fn convert_keys_from_b64(input: &Vec<AgentPubKeyB64>) -> Vec<AgentPubKey> {
    input.iter().map(|k| AgentPubKey::from(k.clone())).collect()
}

// function copied from connors acorn project
// extracts entry, but checks on header type
// is very helpful for catching errors while mocking
// or when mixing up hashes
pub fn entry_from_element_create_or_update<
    E: TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    element: &Element,
) -> Result<E, Error> {
    debug!("utils: entry extraction");
    match element.header() {
        Header::Create(_) | Header::Update(_) => match element.entry().to_app_option() {
            Ok(Some(entry)) => Ok(entry),
            Ok(None) => Err(Error::EntryMissing),
            Err(e) => return Err(Error::Wasm(e.into())),
        },
        _ => {
            debug!("which header {:?}", element.header());
            Err(Error::WrongHeader)
        }
    }
}

pub fn entry_hash_from_element(element: &Element) -> ExternResult<&EntryHash> {
    debug!("utils: entry hash extraction");
    match element.header() {
        Header::Create(_) | Header::Update(_) => match element.header().entry_hash() {
            Some(entry_hash) => Ok(entry_hash),
            None => Err(WasmError::Guest("no entry hash".into())),
        },
        _ => {
            debug!("which header {:?}", element.header());
            Err(WasmError::Guest("WrongHeader".into()))
        }
    }
}

pub fn enable_tracing(level: tracing::Level) {
    // i have no idea where to put the tracing config, as all examples suggest main
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
