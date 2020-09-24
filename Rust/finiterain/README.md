# Finite Rain

## Description of the algorithm

This algorithm is quite simple and slow. I decided to implement something straightforward as the first version, the implementation is in file `src/slow.rs`.

- In the examples of input data the landscape contains at most tens of items. Simple implementation can be acceptable and sometimes it can be the fastest one because of cache localities on modern CPU.
- Such implementation can help with development of faster algorithm: it can be used in unit tests to compare output of slow but correct algorithm and an output of faster algorithm for the same inputs. The test can use predefined or random input data. Some people don't like random inputs in tests, because they can make failures replication more difficult. Other people like them, because they can help to reveal unexpected edge cases.
- This algorithm recalculates already known data in every iteration, so there is definitively faster version.
- To avoid problems with floating point numbers I used fractional numbers. They are slower, but avoids rounding errors for this task.
- The implementation adds one high segment to the very right and very left of the landscape, to make iterations over landscape easier.

The algorithm iterates over remaining time (== amount of water that falls on each segment). In every iteration, it finds local minimas and maximas in the landscape. It works with the idea that water flows to local minimas and don't stay in higher areas. If some segment is a local maxima, then water flows to both its sides in the same amount. If it is neither part of local minama nor local maximam all its water flows down towards some local minima. This behavior is obserbed in the nature and is taken as an axiom.

Algorithm calculates number of water that flows to every local minima and what local minima will be filled at first. If remaining time is shorter than the time that is necessary to fill that local minima, it re-calculates water levels in every local minima and stops. Otherwise it re-calculates water levels, re-scan the landscape (water may flow partly in different directions because at least one minima is filled), subtract the time from remaining time and continues. The full re-scanning is obvious vaste of CPU time, because it would be necessary to re-calculate flows only in part of the landscape around the filled local minima(s) (there can be more filled minimas).

This alforithm has worst case time complexity `O(w*(max_h - min_h))`. Where `w` is the width of the landscape (number of segments),  `h_max` is height of the biggest segment and `h_min` of the smallest segment. This is because this algorihm fills at least one level of at least one local minima (that has at least one segment) in every iteration. While time and water levels can be fractional numbers, at least one level is always filled and that level is expessed as a whole number and there is limited number of such levels.

## Tests

Tests were used during development, to run them, you can use `cargo`:

```
finiterain $ cargo test
```

## Running of the app

You can run the app with `cargo`, first parameter is the number of hours rain is falling (amount of water that falls to every segment), other parameters are segment sizes. Segment size sgould fit to `u32`.

```
finiterain $ cargo run 2 1 5 5
    ...
> rain_amount: 2; landscape: 1, 5, 5;
> Result: 17/3, 17/3, 17/3;
```

