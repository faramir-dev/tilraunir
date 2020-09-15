use std::num::ParseIntError;
use std::result::Result;
use std::vec::Vec;
use structopt::StructOpt;

fn parse_i64_as_u32(s: &str) -> Result<i64, ParseIntError> {
    let num = s.parse::<u32>()?;
    Ok(num as i64)
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "finiterain",
    about = "Calculate water levels after rain falls uniformly on a landscape."
)]
pub(crate) struct Config {
    #[structopt(parse(try_from_str = parse_i64_as_u32))]
    pub(crate) total_time: i64,
    #[structopt(required = true, parse(try_from_str = parse_i64_as_u32))]
    pub(crate) landscape: Vec<i64>,
}
