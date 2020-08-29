use anyhow;
use std::env;
use std::result::Result;
use std::vec::Vec;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub(crate) enum Error {
    #[error("Expected amount of rain and at least one landscape segment")]
    ArgsTooShort,
    #[error("Cannot parse amount of rain, expected non-negative integer, found: {found:?}")]
    InvalidAmountOfRain { found: String },
    #[error("Cannot parse landscape segment, expected non-negative integer, found: {found:?}")]
    InvalidSegment { found: String },
}

pub(crate) struct Config {
    pub(crate) rain_amount: i64,
    pub(crate) landscape: Vec<i64>,
}

pub(crate) fn load() -> anyhow::Result<Config> {
    let rain_amount = env::args()
        .nth(1)
        .ok_or(Error::ArgsTooShort)
        .and_then(|s| {
            s.parse::<i64>()
                .ok()
                .and_then(|x| if x > 0 { Some(x) } else { None })
                .ok_or(Error::InvalidAmountOfRain { found: s })
        })?;
    let landscape = env::args()
        .skip(2)
        .map(|s| {
            s.parse::<u32>()
                .map_err(|_| Error::InvalidSegment { found: s })
                .map(|x| x as i64)
        })
        .collect::<Result<Vec<i64>, _>>()?;
    if landscape.is_empty() {
        Err(Error::ArgsTooShort)?;
    }
    Ok(Config {
        rain_amount,
        landscape,
    })
}
