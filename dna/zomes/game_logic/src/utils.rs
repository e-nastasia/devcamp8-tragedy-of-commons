use hdk::prelude::*;

/// Tries to do a DHT get to retrieve data for the entry_hash,
/// and if this get is successful and we get some element, tries
/// to convert this element into a type T and return the result
pub fn try_get_and_convert<T: TryFrom<Entry>>(entry_hash: EntryHash) -> ExternResult<T> {
    match get(entry_hash.clone(), GetOptions::default())? {
        Some(element) => try_from_element(element),
        None => Err(WasmError::Guest(
            format!("There is no element at the hash {}", entry_hash),
        )),
    }
}

/// Tries to extract the entry from the element, and if the entry is there
/// tries to convert it to type T and return the result
pub fn try_from_element<T: TryFrom<Entry>>(element: Element) -> ExternResult<T> {
    match element.entry() {
        element::ElementEntry::Present(entry) => {
            // NOTE(e-nastasia): would be cool to rewrite this error to include name of T into the message
            // I think we'd need to add more trait requirements in the fn signature to guarantee that every
            // T has some fn to call that returns it's pretty name for messages.
            T::try_from(entry.clone()).or(Err(WasmError::Guest(format!("Couldn't convert Element entry {:?} into provided data type", entry))))
        }
        _ => Err(WasmError::Guest(format!("Element {:?} does not have an entry", element))),
    }
}
