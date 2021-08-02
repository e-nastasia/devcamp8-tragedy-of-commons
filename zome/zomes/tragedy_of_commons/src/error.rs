use hdk::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Element with invalid header")]
    WrongHeader,

    #[error("Element is missing its Entry")]
    EntryMissing,

    #[error("Element is missing Entry hash")]
    EntryHashMissing,

    #[error("Wasm Error {0}")]
    Wasm(WasmError),
}

impl From<Error> for WasmError {
    fn from(e: Error) -> Self {
        WasmError::Guest(e.to_string())
    }
}

impl From<Error> for ValidateCallbackResult {
    fn from(e: Error) -> Self {
        ValidateCallbackResult::Invalid(e.to_string())
    }
}

impl From<Error> for ExternResult<ValidateCallbackResult> {
    fn from(e: Error) -> Self {
        Ok(e.into())
    }
}
