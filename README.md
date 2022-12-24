# My Advent of Code solutions, implemented in Rust

Each package in the workspace represents the set of puzzles for some year.
Puzzle binaries are located at src/bin/ for each year's package. You need to
provide the corresponding day's input file for each puzzle binary.

You can run any inplemented puzzle solution using the following format:

```sh
cargo run --package y`year` --bin day`day`_`part` `path`
```

For example, day 1 puzzle, for the year 2022, part 1:

```sh
cargo run --package y2022 --bin day01_1 day01
```

## Where can I get the puzzle texts and inputs?

From Advent of Code website directly. Puzzle texts and inputs are not licensed for
reproduction or distribution. See the [legal notice](https://adventofcode.com/about)
on adventofcode.com > About > Legal.
For example, you can get day 1 puzzle text (2022) from: <https://adventofcode.com/2022/day/1>,
and the input from: <https://adventofcode.com/2022/day/1/input>.

## License

MIT licensed.
