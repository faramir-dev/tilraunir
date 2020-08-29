use num_rational::Rational64;
use std::assert;
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
    water_speed: i64, // Maximas on the very right/very left have 0
}

type LocalMaximas = Vec<LocalMaximum>;
type LocalMinimas = Vec<LocalMinimum>;

fn find_extremes(landscape: &Vec<Rational64>) -> (LocalMaximas, LocalMinimas) {
    assert!(landscape.len() >= 3);

    let mut local_maximas: LocalMaximas = Vec::new();
    let mut local_minimas: LocalMinimas = Vec::new();

    let find_minimum = |from: usize| -> LocalMinimum {
        assert!(from + 1 < landscape.len());

        let mut min_val = landscape[from];
        let mut begin = from;
        let mut end = from + 1;
        loop {
            let val = landscape[end];
            if val < min_val {
                begin = end;
                min_val = val;
            } else if val > min_val {
                break;
            }
            end = end + 1;
        }
        LocalMinimum {
            begin,
            width: end - begin,
            depth: std::cmp::min(landscape[begin - 1] - min_val, landscape[end] - min_val),
        }
    };
    let find_maximum = |from: usize| -> LocalMaximum {
        assert!(from < landscape.len());

        let mut max_val = landscape[from];
        let mut begin = from;
        let mut end = from + 1;
        loop {
            assert!(end <= landscape.len());
            if end >= landscape.len() {
                break;
            }
            let val = landscape[end];
            if val > max_val {
                begin = end;
                max_val = val;
            } else if val < max_val {
                break;
            }
            end += 1;
        }
        let water_speed = if end < landscape.len() {
            (end - begin) as i64
        } else {
            0
        };
        LocalMaximum {
            begin,
            width: end - begin,
            water_speed,
        }
    };

    local_maximas.push(LocalMaximum {
        begin: 0,
        width: 1,
        water_speed: 0,
    });

    let mut from = 1;
    while from < landscape.len() {
        let minimum = find_minimum(from);
        from = minimum.begin + minimum.width;
        local_minimas.push(minimum);

        let maximum = find_maximum(from);
        from = maximum.begin + maximum.width;
        local_maximas.push(maximum);
    }
    (local_maximas, local_minimas)
}

#[test]
fn test_find_extremes() {
    {
        let landscape = vec![
            Rational64::from_integer(5),
            Rational64::from_integer(2),
            Rational64::from_integer(5),
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
                depth: Rational64::from_integer(3)
            }]
        );
    }

    {
        let landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
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
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(2),
            Rational64::from_integer(2),
            Rational64::from_integer(2),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
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

pub(crate) fn calculate(total_time: Rational64, landscape: &mut Vec<Rational64>) {
    assert!(landscape.len() >= 3);
    assert!(landscape[0] == crate::MAX);
    assert!(landscape[landscape.len() - 1] == crate::MAX);

    let mut remaining_time = total_time;
    while remaining_time > Rational64::from_integer(0) {
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
fn test_calculate() {
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(1),
            crate::MAX,
        ];
        calculate(Rational64::from_integer(10), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(11),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            crate::MAX,
        ];
        calculate(Rational64::from_integer(11), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(16),
                Rational64::from_integer(16),
                Rational64::from_integer(16),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(1),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            crate::MAX,
        ];
        calculate(Rational64::from_integer(1), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(4),
                Rational64::from_integer(5),
                Rational64::from_integer(5),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(1),
            Rational64::from_integer(5),
            Rational64::from_integer(5),
            crate::MAX,
        ];
        calculate(Rational64::from_integer(2), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::new(17, 3),
                Rational64::new(17, 3),
                Rational64::new(17, 3),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(1),
            Rational64::from_integer(9),
            Rational64::from_integer(1),
            crate::MAX,
        ];
        calculate(Rational64::from_integer(1), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::new(5, 2),
                Rational64::from_integer(9),
                Rational64::new(5, 2),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::new(1, 2), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(3),
                Rational64::new(9, 4),
                Rational64::from_integer(6),
                Rational64::new(23, 4),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::new(4, 7), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(3),
                Rational64::new(17, 7),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::new(2, 3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(3),
                Rational64::from_integer(3),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::new(5, 3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(6),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::from_integer(3), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(8),
                Rational64::from_integer(9),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::new(23, 6), &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                Rational64::from_integer(9),
                crate::MAX,
            ]
        );
    }
    {
        let mut landscape = vec![
            crate::MAX,
            Rational64::from_integer(3),
            Rational64::from_integer(1),
            Rational64::from_integer(6),
            Rational64::from_integer(4),
            Rational64::from_integer(8),
            Rational64::from_integer(9),
            crate::MAX,
        ];
        calculate(Rational64::new(23, 6) + 11, &mut landscape);
        assert_eq!(
            &landscape[..],
            &[
                crate::MAX,
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                Rational64::from_integer(20),
                crate::MAX,
            ]
        );
    }
}
