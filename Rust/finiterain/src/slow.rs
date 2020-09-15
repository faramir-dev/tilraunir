use crate::{MAX, ZERO};
use num_rational::Rational64;
use std::assert;
use std::cmp::min;
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
struct LocalMinimum {
    begin: usize,
    width: usize,
    depth: Rational64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LocalMaximum {
    begin: usize,
    width: usize,
    water_speed: i64, // 0 for maximas on the very right/very left, `width` otherwise
}

type LocalMaximas = Vec<LocalMaximum>;
type LocalMinimas = Vec<LocalMinimum>;

fn find_extremes(landscape: &[Rational64]) -> (LocalMaximas, LocalMinimas) {
    let len = landscape.len();
    let mut local_maximas: LocalMaximas = Vec::new();
    let mut local_minimas: LocalMinimas = Vec::new();

    let find_minimum = |from: usize| -> LocalMinimum {
        assert!(from + 2 < len);

        let mut begin = from;
        let mut end = from + 1;
        while landscape[begin] >= landscape[end] {
            if landscape[begin] > landscape[end] {
                begin = end;
            }
            end += 1;
        }
        let min_val = landscape[begin];
        LocalMinimum {
            begin,
            width: end - begin,
            depth: min(landscape[begin - 1], landscape[end]) - min_val,
        }
    };
    let find_maximum = |from: usize| -> LocalMaximum {
        assert!(from + 1 < len);

        let mut begin = from;
        let mut end = from + 1;
        while landscape[begin] <= landscape[end] {
            if landscape[begin] < landscape[end] {
                begin = end;
            }
            end += 1;
        }
        let width = end - begin;
        let water_speed = if end + 1 < len { width as i64 } else { 0 };
        LocalMaximum {
            begin,
            width,
            water_speed,
        }
    };

    local_maximas.push(LocalMaximum {
        begin: 0,
        width: 1,
        water_speed: 0,
    });

    let mut from = 1;
    while from + 1 < len {
        let minimum = find_minimum(from);
        from = minimum.begin + minimum.width;
        local_minimas.push(minimum);

        let maximum = find_maximum(from);
        from = maximum.begin + maximum.width;
        local_maximas.push(maximum);
    }
    (local_maximas, local_minimas)
}

fn find_water_speeds(maximas: &LocalMaximas, minimas: &LocalMinimas) -> Vec<Rational64> {
    (0..minimas.len())
        .map(|i| {
            let left = &maximas[i];
            let right = &maximas[i + 1];
            let left_end = left.begin + left.width;
            Rational64::from_integer(left.water_speed) / 2
                + Rational64::from_integer(right.water_speed) / 2
                + (right.begin - left_end) as i64
        })
        .collect()
}

pub(crate) fn calculate(total_time: Rational64, landscape: &mut [Rational64]) {
    let len = landscape.len();
    assert!(len >= 4);
    assert!(landscape[0] == MAX);
    assert!(landscape[len - 2] == MAX);
    assert!(landscape[len - 1] == ZERO);

    let mut remaining_time = total_time;
    while remaining_time > ZERO {
        let (maximas, minimas) = find_extremes(landscape);
        let speeds = find_water_speeds(&maximas, &minimas);
        let min_time = speeds
            .iter()
            .zip(minimas.iter())
            .map(|(speed, minima)| minima.depth * minima.width as i64 / speed)
            .min()
            .unwrap();
        let step_time = std::cmp::min(min_time, remaining_time);

        remaining_time -= step_time;
        for idx in 0..minimas.len() {
            let speed = speeds[idx];
            let add = speed * step_time / minimas[idx].width as i64;
            let b = minimas[idx].begin;
            let e = b + minimas[idx].width;
            for i in b..e {
                landscape[i] += add;
            }
        }
    }
}

#[test]
fn test_find_extremes() {
    {
        let landscape = vec![MAX, Rational64::from_integer(2), MAX, ZERO];
        let (maximas, minimas) = find_extremes(&landscape);
        assert_eq!(
            &maximas[..],
            &[
                LocalMaximum {
                    begin: 0,
                    width: 1,
                    water_speed: 0
                },
                LocalMaximum {
                    begin: 2,
                    width: 1,
                    water_speed: 0
                }
            ]
        );
        assert_eq!(
            &minimas[..],
            &[LocalMinimum {
                begin: 1,
                width: 1,
                depth: MAX - 2
            }]
        );
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
        let (maximas, minimas) = find_extremes(&landscape);
        assert_eq!(
            &maximas[..],
            &[
                LocalMaximum {
                    begin: 0,
                    width: 1,
                    water_speed: 0
                },
                LocalMaximum {
                    begin: 3,
                    width: 1,
                    water_speed: 1
                },
                LocalMaximum {
                    begin: 7,
                    width: 1,
                    water_speed: 0
                }
            ]
        );
        assert_eq!(
            &minimas[..],
            &[
                LocalMinimum {
                    begin: 2,
                    width: 1,
                    depth: Rational64::from_integer(2),
                },
                LocalMinimum {
                    begin: 4,
                    width: 1,
                    depth: Rational64::from_integer(2)
                }
            ]
        );
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
        let (maximas, minimas) = find_extremes(&landscape);
        assert_eq!(
            &maximas[..],
            &[
                LocalMaximum {
                    begin: 0,
                    width: 1,
                    water_speed: 0
                },
                LocalMaximum {
                    begin: 3,
                    width: 1,
                    water_speed: 1
                },
                LocalMaximum {
                    begin: 9,
                    width: 1,
                    water_speed: 0
                }
            ]
        );
        assert_eq!(
            &minimas[..],
            &[
                LocalMinimum {
                    begin: 2,
                    width: 1,
                    depth: Rational64::from_integer(2),
                },
                LocalMinimum {
                    begin: 4,
                    width: 3,
                    depth: Rational64::from_integer(4)
                }
            ]
        );
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
