# Dobutsu Shogi Analyzer (2024)

The solver perfectly solves the game of Dobutsu Shogi.
You can use it to find the best move in any reachable position.
You can try it out [here](https://kylejlin.github.io/dobutsu_shogi_2024).

> Note: The name of this repository is qualified with "2024" to distinguish it from
> a similar solver I wrote in 2021.

This repository contains two parts: the core solver (written in Rust), and a frontend (written in TypeScript).
The solver generates a database of all reachable states and their respective optimal actions.
The frontend uses this database to analyze positions and find the best move.

## Creating the database

```sh
git clone https://github.com/kylejlin/dobutsu_shogi_2024.git

cd dobutsu_shogi_2024

cargo test --release

cargo run --release

simpledb
```

When you run `cargo run --release`, the solver first computes the database.
This may take several hours (or even days).
Once it finishes, you should see a prompt that says something like

```txt
Tree inspector ready. Type \"launch\" to launch or \"simpledb\" to create a simple best-child database.
Launching will clear the console, so be sure to save any important information.
```

Once you see this prompt, type `simpledb` and press Enter.
The solver will save the database to the `db` directory.

## Using the database

### Database structure

Conceptually, the database is a dictionary that maps each non-terminal game state to its respective optimal child state.
Each child state is annotated with its evaluation (Win in `n`, Loss in `n`, or Draw).

We store the database as a list of annotated child states.
In theory, you can use this list to look up the optimal child state for any non-terminal game state `s`.
Specifically, you first compute all of `s`'s children (let's call this list `c`).
Then, you iterate over the database, and for each entry `e`, you check if `e` is in `c`.
Out of all the entries in `c`, you choose the one with the lowest evaluation.

In practice, this is not feasible because the database is too large.
So, we sort the list based on the order of

Conveniently, since states are [represented as bit arrays](./docs/spec.md#state-representation-40-bits-total), we can trivially define total order on them.

We store the database in many files, each containing a list of annotated child states.

## License (MIT)

Copyright (c) 2024 Kyle Lin

This repository is licensed under the MIT license.
