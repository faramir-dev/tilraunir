#![forbid(unsafe_code)]

mod config;
mod slow;

use anyhow;
use num_rational::Rational64;
use std::fmt;
use std::vec::Vec;

const MAX: Rational64 = Rational64::new_raw(std::u32::MAX as i64 * 2, 1);
const ZERO: Rational64 = Rational64::new_raw(0, 1);

struct Fmt<'a, T: fmt::Display>(&'a [T]);
impl<T: fmt::Display> fmt::Display for Fmt<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
        }
        Ok(())
    }
}

fn main() {
    let maybe_err = || -> anyhow::Result<()> {
        let cfg = config::load()?;
        println!(
            "> total_time: {}; landscape: {};",
            cfg.total_time,
            Fmt(&cfg.landscape[1..])
        );

        let mut landscape = vec![MAX];
        landscape.extend(
            cfg.landscape
                .iter()
                .map(|x| Rational64::from_integer(*x))
                .collect::<Vec<Rational64>>(),
        );
        // Add one local maximum to the end of the landscape to simplify iterations.
        landscape.push(MAX);
        landscape.push(ZERO);

        let time = Rational64::from_integer(cfg.total_time);
        slow::calculate(time, &mut landscape);
        println!("> Result: : {};", Fmt(&landscape[1..landscape.len() - 2]));

        Ok(())
    }();
    if let Err(err) = maybe_err {
        eprintln!("Error occurred: {}", err);
    }
}
