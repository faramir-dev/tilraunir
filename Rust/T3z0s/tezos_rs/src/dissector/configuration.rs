use crypto::hash::HashType;
use failure::Error;
use lazy_static::lazy_static;
use libc::c_char;
use serde::Deserialize;
use std::ffi::CStr;
use std::fs;
use std::option::Option;
use std::sync::Arc;
use std::sync::RwLock;

use hex;

use crate::dissector::logger::msg;

use crate::dissector::error::{
    NotTezosStreamError, PeerNotUpgradedError, TezosNodeIdentityNotLoadedError,
    UnknownDecrypterError,
};

static DEFAULT_IDENTITY_FILEPATH: &'static str = "identity/identity.json";

#[derive(Deserialize, Clone, Debug, PartialEq)]
/// Node identity information
pub struct Identity {
    pub peer_id: String,
    pub public_key: String,
    pub secret_key: String,
    pub proof_of_work_stamp: String,
}

#[derive(Debug, Clone)]
/// Dissector configuration
pub(crate) struct Config {
    pub identity_json_filepath: String,
    pub identity: Identity, // As loaded from identity_json_filepath
}
impl Config {
    fn default() -> Result<Self, Error> {
        Ok(Self {
            identity_json_filepath: String::from(DEFAULT_IDENTITY_FILEPATH),
            identity: load_identity(&DEFAULT_IDENTITY_FILEPATH)?,
        })
    }
}

// Configuration is stored in global object.
lazy_static! {
    static ref CONFIG: RwLock<Arc<Result<Config, Error>>> =
        RwLock::new(Arc::new(Err(TezosNodeIdentityNotLoadedError.into())));
}

/// Load identity from given file path
pub fn load_identity(filepath: &str) -> Result<Identity, Error> {
    let content = fs::read_to_string(filepath)?;
    let mut identity: Identity = serde_json::from_str(&content)?;
    let decoded = hex::decode(&identity.public_key)?;
    identity.public_key = HashType::CryptoboxPublicKeyHash.bytes_to_string(&decoded);
    Ok(identity)
}

/// Load identity from file whose path is stored in C string
fn load_preferences(identity_json_filepath: *const c_char) -> Result<Config, Error> {
    if identity_json_filepath.is_null() {
        // Interpret C NULL as a Rust Error
        Err(TezosNodeIdentityNotLoadedError)?;
    }

    let identity_json_filepath =
        unsafe { CStr::from_ptr(identity_json_filepath).to_str()?.to_owned() };

    let identity = load_identity(&identity_json_filepath)?;

    Ok(Config {
        identity_json_filepath,
        identity,
    })
}

#[no_mangle]
/// Called by Wireshark when module preferences change
pub extern "C" fn tezos_preferences_update(identity_json_filepath: *const c_char) {
    let new_cfg = load_preferences(identity_json_filepath);
    let mut cfg = CONFIG.write().unwrap();
    *cfg = Arc::new(new_cfg);
}

pub(crate) fn with_configuration<F, R>(f: F) -> R
where
    F: FnOnce(&Result<Config, Error>) -> R,
{
    msg(format!("with_configuration"));
    let cfg = CONFIG.read().unwrap().clone();
    f(&*cfg)
}
