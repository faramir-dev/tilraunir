mod config;
mod slow;

use anyhow;
use num_rational::Rational64;
use std::vec::Vec;

const MAX: Rational64 = Rational64::new_raw(std::u32::MAX as i64 * 2, 1);

fn main() {
    let maybe_err = || -> anyhow::Result<()> {
        let cfg = config::load()?;
        println!(
            "> rain_amount: {}; landscape: {};",
            cfg.rain_amount,
            cfg.landscape[1..]
                .iter()
                .fold(cfg.landscape[0].to_string(), |acc, &num| acc
                    + ", "
                    + &num.to_string())
        );

        let mut landscape = vec![MAX];
        landscape.extend(
            cfg.landscape
                .iter()
                .map(|x| Rational64::from_integer(*x))
                .collect::<Vec<Rational64>>(),
        );
        landscape.push(MAX);

        let time = Rational64::from_integer(cfg.rain_amount);
        slow::calculate(time, &mut landscape);
        println!(
            "> Result: : {};",
            landscape[2..landscape.len() - 1]
                .iter()
                .fold(landscape[1].to_string(), |acc, &num| acc
                    + ", "
                    + &num.to_string())
        );

        Ok(())
    }();
    if let Err(err) = maybe_err {
        eprintln!("Error occurred: {}", err);
    }
}
