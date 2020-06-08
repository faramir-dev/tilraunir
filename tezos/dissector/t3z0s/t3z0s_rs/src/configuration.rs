use lazy_static::lazy_static;
use libc::{c_char, c_int, c_uint, c_void};
use serde::Deserialize;
use std::fs;
use std::ffi::CStr;
use std::ffi::OsString;
use std::sync::RwLock;
use std::option::Option;
use failure::Error;

use crate::logger::msg;

static DEFAULT_IDENTITY_FILEPATH:&'static str = "identity/identity.json";

#[derive(Deserialize, Clone, Debug, PartialEq)]
/// Node identity information
pub struct Identity {
    pub peer_id: String,
    pub public_key: String,
    pub secret_key: String,
    pub proof_of_work_stamp: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub identity_json_filepath: String,
    pub identity: Identity,
}
impl Config {
    fn default() -> Result<Self, Error> {
        Ok(Self {
            identity_json_filepath: String::from(DEFAULT_IDENTITY_FILEPATH),
            identity: load_identity(&DEFAULT_IDENTITY_FILEPATH)?,
        })
    }
}

lazy_static! {
    static ref config_rwlock: RwLock<Option<Config>> = RwLock::new(None);
}

// https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
// https://stackoverflow.com/questions/55977067/how-can-i-create-a-static-string-in-rust
// https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi

pub fn load_identity(filepath: &str) -> Result<Identity, Error> {
    let content = fs::read_to_string(filepath)?;
    Ok(serde_json::from_str(&content)?)
}

fn load_preferences(identity_json_filepath: *const c_char) -> Result<Config, Error> {
    let identity_json_filepath = unsafe {
        CStr::from_ptr(identity_json_filepath).to_str()?.to_owned()
    };

    let identity = load_identity(&identity_json_filepath)?;

    Ok(Config{identity_json_filepath, identity})
}

#[no_mangle]
pub extern "C" fn t3z0s_preferences_update(identity_json_filepath: *const c_char) {
    if identity_json_filepath.is_null() {
        let mut cfg = config_rwlock.write().unwrap();
        *cfg = None;
    } else {
        let cfg_res = load_preferences(identity_json_filepath);
        let mut cfg = config_rwlock.write().unwrap();
        *cfg = match cfg_res {
            Ok(new_cfg) => Some(new_cfg),
            Err(e) => { msg(format!("Cannot load configuration: {}", e)); None }
        }
    }
}

pub(crate) fn get_configuration() -> Option<Config> {
    let cfg = config_rwlock.read().unwrap();
    // TODO: Unecessary clone, maybe use shared-ptr?
    cfg.clone()
}