use crate::{MAX, ZERO};
use num_rational::Rational64;
use std::assert;
use std::cmp::min;
use std::ops::Range;
use std::vec::Vec;

type LocalMinimum = Range<usize>;
type LocalMaximum = Range<usize>;
type LocalMaximas = Vec<LocalMaximum>;
type LocalMinimas = Vec<LocalMinimum>;

fn calculate_extremes(landscape: &[Rational64]) -> (LocalMaximas, LocalMinimas) {
    let mut local_maximas: LocalMaximas = Vec::new();
    let mut local_minimas: LocalMinimas = Vec::new();

    local_maximas.push(0..0);
    let mut prev = (0, MAX);
    let mut curr = (0, landscape[0]);
    loop {
        match (curr.0 + 1..landscape.len()).find(|&i| landscape[i] != curr.1) {
            Some(next_idx) => {
                let next = (next_idx, landscape[next_idx]);
                if prev.1 > curr.1 && curr.1 < next.1 {
                    local_minimas.push(curr.0..next.0);
                } else if prev.1 < curr.1 && curr.1 > next.1 {
                    local_maximas.push(curr.0..next.0);
                }
                prev = curr;
                curr = next;
            }
            None => break,
        };
    }
    if prev.1 > curr.1 {
        local_minimas.push(curr.0..landscape.len());
    }
    local_maximas.push(landscape.len()..landscape.len());

    (local_maximas, local_minimas)
}

fn calculate_water_currents(maximas: &LocalMaximas) -> Vec<Rational64> {
    let mut ret = Vec::new();
    let mut left = &maximas[0];
    let mut left_current = ZERO;
    for i in 1..maximas.len() {
        let right = &maximas[i];
        let right_current = Rational64::from_integer(right.len() as i64) / 2;
        ret.push(left_current + right_current + (right.start - left.end) as i64);
        left = &right;
        left_current = right_current;
    }
    ret
}

pub(crate) fn calculate(total_time: Rational64, landscape: &mut [Rational64]) {
    let len = landscape.len();
    assert!(len >= 1);

    let mut remaining_time = total_time;
    while remaining_time > ZERO {
        let (maximas, minimas) = calculate_extremes(landscape);
        let currents = calculate_water_currents(&maximas);
        let depths_iter = (0..minimas.len()).map(|i| {
            let start = minimas[i].start;
            let end = minimas[i].end;
            let min_val = landscape[start];
            let left_depth = if start > 0 {
                landscape[start - 1] - min_val
            } else {
                MAX
            };
            let right_depth = if end < len {
                landscape[end] - min_val
            } else {
                MAX
            };
            min(left_depth, right_depth)
        });
        let min_time = currents
            .iter()
            .zip(minimas.iter())
            .zip(depths_iter)
            .map(|((current, minima), depth)| depth * minima.len() as i64 / current)
            .min()
            .unwrap();
        let step_time = min(min_time, remaining_time);

        remaining_time -= step_time;
        for idx in 0..minimas.len() {
            let current = currents[idx];
            let add = current * step_time / minimas[idx].len() as i64;
            let b = minimas[idx].start;
            let e = minimas[idx].end;
            for i in b..e {
                landscape[i] += add;
            }
        }
    }
}

#[test]
fn test_calculate_extremes() {
    {
        let landscape = vec![Rational64::from_integer(2)];
        let (maximas, minimas) = calculate_extremes(&landscape);
        assert_eq!(&maximas[..], &[0..0, 1..1,]);
        assert_eq!(&minimas[..], &[0..1]);
    }

    {
        let landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        let (maximas, minimas) = calculate_extremes(&landscape);
        assert_eq!(&maximas[..], &[0..0, 2..3, 6..6,]);
        assert_eq!(&minimas[..], &[1..2, 3..4,]);
    }

    {
        let landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(2),
            Rational64::from_integer(2),
            Rational64::from_integer(2),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        let (maximas, minimas) = calculate_extremes(&landscape);
        assert_eq!(&maximas[..], &[0..0, 2..3, 8..8,]);
        assert_eq!(&minimas[..], &[1..2, 3..6,]);
    }
}
#[test]
fn test_calculate() {
    {
        let mut landscape = vec![Rational64::from_integer(1)];
        calculate(Rational64::from_integer(10), &mut landscape);
        assert_eq!(&landscape[..], &[Rational64::from_integer(11)]);
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
        ];
        calculate(Rational64::from_integer(11), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(16),
                Rational64::from_integer(16),
                Rational64::from_integer(16),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(1),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
        ];
        calculate(Rational64::from_integer(1), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(4),
                Rational64::from_integer(5),
                Rational64::from_integer(5),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(1),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
        ];
        calculate(Rational64::from_integer(2), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::new(17, 3),
                Rational64::new(17, 3),
                Rational64::new(17, 3),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(1),
            Rational64::from_integer(9),
            Rational64::from_integer(1),
        ];
        calculate(Rational64::from_integer(1), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::new(5, 2),
                Rational64::from_integer(9),
                Rational64::new(5, 2),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::new(1, 2), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(3),
                Rational64::new(9, 4),
                Rational64::from_integer(6),
                Rational64::new(23, 4),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::new(4, 7), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(3),
                Rational64::new(17, 7),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::new(2, 3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(3),
                Rational64::from_integer(3),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::new(5, 3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::from_integer(3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::new(23, 6), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
            ]
        );
    }
    {
        let mut landscape = vec![
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
        ];
        calculate(Rational64::new(23, 6) + 11, &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
            ]
        );
    }
}
