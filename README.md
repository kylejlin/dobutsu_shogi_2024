# Dobutsu Shogi Analyzer (2024)

The solver perfectly solves the game of Dobutsu Shogi.
You can use it to find the best move in any reachable position.
You can try it out [here](https://kylejlin.github.io/dobutsu_shogi_2024).

> Note: The name of this repository is qualified with "2024" to distinguish it from
> a similar solver I wrote in 2021.

This repository contains two parts: the core solver (written in Rust), and a frontend (written in TypeScript).
The solver generates a database of all reachable states and their respective optimal actions.
The frontend uses this database to analyze positions and find the best move.

## Table of contents

1. [Creating the database](#creating-the-database)
2. [Using the database](#using-the-database)
3. [Data representation](#data-representation)
4. [License](#license-mit)

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
This may take several hours (or even days).

From here, it's up to you to decide how you want to host the database.
I personally chose to commit it to a Git repository.
I uploaded this to [GitHub](https://github.com/kylejlin/dobutsu_shogi_database_2024), so I could easily access it from the web.
However, you can also serve it locally, or use a different hosting service.

## Using the database

### Database structure

Conceptually, the database is a dictionary that maps each non-terminal game state to its respective optimal child state.
Each child state is annotated with its evaluation (Win in `n`, Loss in `n`, or Draw).

Conveniently, since states are [represented as bit arrays](./docs/spec.md#state-representation-40-bits-total), we can trivially define total order on them.
Consequently, we can think of the database as a sorted list of parent-child pairs, sorted by the parent.

This list needs to be accessible from the web, so we cannot store it as a single massive file.
Instead, we store the list as many smaller files, numbered `0.dat`, `1.dat`, `2.dat`, etc.
Each file contains a contiguous chunk of the original list, called a _packet_.
To save space, we only save the child of each parent-child pair.

We also store a list of the maximums of the parents of each packet.
We store this in `maximums.dat`.
This lets us find the packet index associated with a given parent state.
For example, suppose `maximums.dat` contains the following numbers:

```txt
21
57
93
124
```

> Note: These numbers are unrealistic. I only chose them for the sake of demonstration.

Suppose we want to find the packet index associated with parent state `68`.
We would find the smallest number in `maximums.dat` that is greater than or equal to `68`.
This number is `93`, which is at index `2` (assuming zero-based indexing).
So, we know to fetch the packet from `2.dat`.

Now you know how to find the packet associated with a given parent state.
In the next section, we explain how to use the packet to compute the optimal child of said parent state.

> A final note: To make things easier for Git and the filesystem, we split the packets into many directories, instead of storing all 1000000+ packets in one directory.
> Each directory stores 1000 packets.
> The directories are named `0`, `1`, `2`, etc.
> For example, packet 123456 would be stored in `./123/456.dat`.

### Using a packet

Suppose we want to find the optimal child of parent state `68`.
In the previous section, we learned how to find the packet associated with `68`.
In the above example, that packet was `2.dat`.

Now suppose `2.dat` contains the following data:

```txt
21 (score = -5)
7 (score = 0)
19 (score = 3)
56 (score = -1)
75 (score = 2)
```

First, we compute the child states of our parent state.
Suppose `68` only has two children: `21` and `56`.

We iterate through the packet, skipping over the children that are not in our list.
So, we skip over `7`, `19`, and `75`.
In other words, we only consider `21` and `56`.

Out of those children, we choose the one with the lowest score.
In this case, that is `21` (which has a score of `-5`).
This child is the optimal child of parent state `68`.

## Running the web app

After you have cloned the repository and `cd`ed into it, run the following commands:

```sh
cd web
npm install
npm start
```

The app is hardcoded to use the database hosted at `https://kylejlin.github.io/dobutsu_shogi_database_2024`.
However, it is quite simple to change the URL to point to your own copy of the database.

## Data representation

The database is stored in a custom binary format.
See [this spec](./docs/spec.md) for more information.
The spec describes the Dobutsu Shogi rules we chose to use, the state representation, and the database format.

## License (MIT)

Copyright (c) 2024 Kyle Lin

This repository is licensed under the MIT license.
