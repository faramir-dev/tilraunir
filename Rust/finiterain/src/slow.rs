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
    let len = landscape.len();
    let mut local_maximas: LocalMaximas = Vec::new();
    let mut local_minimas: LocalMinimas = Vec::new();

    let find_minimum = |from: usize| -> LocalMinimum {
        assert!(from + 2 < len);

        let mut start = from;
        let mut end = from + 1;
        while landscape[start] >= landscape[end] {
            if landscape[start] > landscape[end] {
                start = end;
            }
            end += 1;
        }
        start..end
    };
    let find_maximum = |from: usize| -> LocalMaximum {
        assert!(from + 1 < len);

        let mut start = from;
        let mut end = from + 1;
        while landscape[start] <= landscape[end] {
            if landscape[start] < landscape[end] {
                start = end;
            }
            end += 1;
        }
        start..end
    };

    local_maximas.push(1..1);
    let mut from = 1;
    while from + 1 < len {
        let minimum = find_minimum(from);
        from = minimum.end;
        local_minimas.push(minimum);

        let maximum = find_maximum(from);
        from = maximum.end;
        local_maximas.push(maximum);
    }
    let last_idx = local_maximas.len() - 1;
    let mut last = &mut local_maximas[last_idx];
    last.end = last.start;

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
    assert!(len >= 4);
    assert!(landscape[0] == MAX);
    assert!(landscape[len - 2] == MAX);
    assert!(landscape[len - 1] == ZERO);

    let mut remaining_time = total_time;
    while remaining_time > ZERO {
        let (maximas, minimas) = calculate_extremes(landscape);
        let currents = calculate_water_currents(&maximas);
        let depths_iter = (0..minimas.len()).map(|i| {
            let start = minimas[i].start;
            let end = minimas[i].end;
            let min_val = landscape[start];
            min(landscape[start - 1], landscape[end]) - min_val
        });
        let min_time = currents
            .iter()
            .zip(minimas.iter())
            .zip(depths_iter)
            .map(|((current, minima), depth)| depth * minima.len() as i64 / current)
            .min()
            .unwrap();
        let step_time = std::cmp::min(min_time, remaining_time);

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
        let landscape = vec![MAX, Rational64::from_integer(2), MAX, ZERO];
        let (maximas, minimas) = calculate_extremes(&landscape);
        assert_eq!(&maximas[..], &[1..1, 2..2,]);
        assert_eq!(&minimas[..], &[1..2]);
    }

    {
        let landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        let (maximas, minimas) = calculate_extremes(&landscape);
        assert_eq!(&maximas[..], &[1..1, 3..4, 7..7,]);
        assert_eq!(&minimas[..], &[2..3, 4..5,]);
    }

    {
        let landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(2),
            Rational64::from_integer(2),
            Rational64::from_integer(2),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        let (maximas, minimas) = calculate_extremes(&landscape);
        assert_eq!(&maximas[..], &[1..1, 3..4, 9..9,]);
        assert_eq!(&minimas[..], &[2..3, 4..7,]);
    }
}
#[test]
fn test_calculate() {
    {
        let mut landscape = vec![MAX, Rational64::from_integer(1), MAX, ZERO];
        calculate(Rational64::from_integer(10), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[MAX, Rational64::from_integer(11), MAX, ZERO,]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            MAX,
            ZERO,
        ];
        calculate(Rational64::from_integer(11), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(16),
                Rational64::from_integer(16),
                Rational64::from_integer(16),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(1),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            MAX,
            ZERO,
        ];
        calculate(Rational64::from_integer(1), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(4),
                Rational64::from_integer(5),
                Rational64::from_integer(5),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(1),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            MAX,
            ZERO,
        ];
        calculate(Rational64::from_integer(2), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::new(17, 3),
                Rational64::new(17, 3),
                Rational64::new(17, 3),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(1),
            Rational64::from_integer(9),
            Rational64::from_integer(1),
            MAX,
            ZERO,
        ];
        calculate(Rational64::from_integer(1), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::new(5, 2),
                Rational64::from_integer(9),
                Rational64::new(5, 2),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::new(1, 2), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(3),
                Rational64::new(9, 4),
                Rational64::from_integer(6),
                Rational64::new(23, 4),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::new(4, 7), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(3),
                Rational64::new(17, 7),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::new(2, 3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(3),
                Rational64::from_integer(3),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::new(5, 3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::from_integer(3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::new(23, 6), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                MAX,
                ZERO,
            ]
        );
    }
    {
        let mut landscape = vec![
            MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            MAX,
            ZERO,
        ];
        calculate(Rational64::new(23, 6) + 11, &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                MAX,
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                MAX,
                ZERO,
            ]
        );
    }
}
