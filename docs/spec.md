# Dobutsu Shogi Analyzer Specification

## Table of contents

1. [Official rules](#official-rules)
2. [Try Rule simplification](#try-rule-simplification)
3. [Threefold repetition rule](#threefold-repetition-rule)
4. [Definition of optimal play](#definition-of-optimal-play)
5. [Algorithm](#algorithm)
6. [State with state stats representation](#state-with-state-stats-representation-56-bits-total)
7. [State representation](#state-representation-40-bits-total)
8. [Action representation](#action-representation-7-bits-total)
9. [Board representation](#board-representation-64-bits-total)
10. [Coordinate set representation](#coordinate-set-representation-16-bits-total)

## Official rules

The term "official rules" refers to the [original Japanese rules](https://nekomado.com/wp/wp-content/uploads/2021/12/jp.pdf).

An English translation may be found [here](https://nekomado.com/wp/wp-content/uploads/2021/12/en.pdf).

In case the documents become inaccessible, PDFs can be found in the [`docs`](./) directory of this repository.

## Try Rule simplification

The official Try Rule is replaced by the following rule:

> Let a player's _home row_ be the row that their lion starts the game in.
> If any player begins their turn with their lion
> in the opponent's home row, that player immediately wins the game.

This captures the spirit of the official Try Rule,
while eliminating edge cases.
The official rule states that a player wins if
they move their lion into the opponent's home row,
except if the opponent immediately captures the lion on their next turn then the opponent wins instead.
The official rule requires more care to implement,
because...

1. When a lion moves into the opposing home row,
   we need to see if it is capturable.
   If not, it is an immediate win for the lion's player.
2. If it is capturable, the game continues.
3. If the opponent captures the lion on their next turn,
   the opponent wins.
   If they make any other move, the lion's player wins.

Instead of these three steps, the new rule is simpler:

1. When a player starts their turn, check to
   see if their lion is in the opponent's home row.
   If so, that player wins.

**Note that this simplification preserve the correctness and optimality of winning and strategies.**

That is:

```
∀ player ∈ {sente, gote}:
  ∀ gameState ∈ GameState:
    ∀ n ∈ ℕ:
      HasWinningStrategyUnderOfficialRules(player, gameState, n) ⇔
        ∃ k ∈ {-1, 0, 1}:
          HasWinningStrategyUnderSimplifiedRules(player, gameState, n + k)
```

The reason we need the `k` term is because this simplification to the Try Rule may prolong the game by one ply in some cases.
This is because the official Try Rule would lead to an immediate win upon scoring the try (provided the scoring lion is not immediately capturable), but the simplified Try Rule would require the opponent to make a move before the game ends.

## Threefold repetition rule

We ignore threefold repetition, and instead allow games to continue indefinitely.
This naturally raises questions about whether this will impact the solver's correctness and ease of implementation.

### Correctness

This simplification preserves the correctness of the solver.
This is because any strategy that wins in the least amount of plies will not contain any duplicate states.

### Implications for implementation

In a naive depth-first search, allowing repetition would almost certainly lead to infinite recursion.
However, we precompute and cache the set of all reachable states first (see [Algorithm](#algorithm) for details).
Since there are a finite number of states, we can guarantee that the search will terminate.

Only after we have precomputed the set of all reachable states do we calculate the best outcome for each state.
We backtrack from winning and losing states, so that draws (which are the only possible kind of divergent state) are never visited.

## Definition of optimal play

For a given state `P`, an optimal action is an action that will lead to a final game state `S` that maximizes the objective function `F`, assuming that both players play optimally in subsequent plies.

The objective function `F` is defined as follows:

1. If a `S.winner == Some(P.activePlayer)`, then `F` is `201 - S.plyCount`.
2. If a `S.winner == Some(P.passivePlayer)`, then `F` is `-201 + S.plyCount`.
3. If a `S.winner == None` (e.g., the game is a draw), then `F` is `0`.

More intuitively, it means players will try to win as quickly as possible, and if they can't win, they will try to delay the opponent's win as long as possible.

## Algorithm

We solve the game in two steps:

1. We calculate the set of all reachable states.
2. We calculate the best outcome for each state
   using retrograde analysis.

   That is, we create a set of states with known outcomes.
   We initialize the set with terminal states.
   Then, we iterate over the set of states with known outcomes,
   and update each state's parents' best known outcomes.
   When all of a parent's children have been visited,
   the parent's best known outcome equals the best known outcome
   (since there are no more children to explore).
   Thus, we add the parent to the set of states with known outcomes.

## State with state stats representation (56 bits total)

| stateStats | state   |
| ---------- | ------- |
| 16 bits    | 40 bits |

> Note: The leftmost column contains the most significant bit.
> Going forward, we will use this convention, except when explicitly stated otherwise.

- `stateStats`: see [State stats representation](#state-stats-representation)
- `state`: see [State representation](#state-representation)

## State stats representation (16 bits total)

| requiredChildReportCount | bestKnownOutcome |
| ------------------------ | ---------------- |
| 7 bits                   | 9 bits           |

- `requiredChildReportCount`: This is an unsigned 7-bit integer that represents the number of children that need to report their outcomes before this state's outcome can be calculated.

  When this becomes zero, `bestKnownOutcome` is the true theoretical best outcome.

  Note that this value does not necessarily decrement one-by-one--if a child has reported a loss,
  we immediately record the parent as a win and set `requiredChildReportCount` to zero.
  There is no need to wait for the other children to report their outcomes
  because there can be no better outcome than a win in the least number of plies.
  Assuming that we use a queue instead of a stack to explore states,
  and that we enqueue all the terminal states first,
  the first-in-first-out nature guarantees that by the time we dequeue a states
  with an outcome in `n` plies, all the states with outcomes in `m` plies where `m < n`
  have already been dequeued.
  Thus, if we dequeue such a states, and the states is a loss in `n` plies,
  we can safely record all non-finalized parents (i.e., every parent with a non-zero `requiredChildReportCount`) as a win in `n + 1` plies,
  since there cannot be a faster win for that parent.

- `bestKnownOutcome`: This is a two's complement 9-bit signed integer that represents the best known outcome of the state.

  - `0` represents a draw.
  - A positive number `n` represents a win for the active player
    in `201 - n` plies.
  - A negative number `-n` represents a win for the passive player
    in `201 + n` plies.

## State representation (40 bits total)

A state stores the positions, allegiances, and promotion statuses of the 8 pieces.
Most pieces require 5 bits (1 bit for allegiance, 4 bits for position).
However:

- The 2 lions do not have an allegiance bit, since lions cannot change allegiance. Therefore, lions require 4 bits each.
- The chicks have an additional promotion bit (since they are the only piece that can be promoted). Therefore, chicks require 6 bits each.

| chick0 | chick1 | elephant0 | elephant1 | giraffe0 | giraffe1 | lionActive | lionPassive |
| ------ | ------ | --------- | --------- | -------- | -------- | ---------- | ----------- |
| 6 bits | 6 bits | 5 bits    | 5 bits    | 5 bits   | 5 bits   | 4 bit      | 4 bit       |

The format for each piece's state is:

| allegiance (if applicable) | row    | column | promotion (if applicable) |
| -------------------------- | ------ | ------ | ------------------------- |
| 1 bit                      | 2 bits | 2 bits | 1 bit                     |

- `allegiance`: The allegiance bit is `0` if the piece belongs to the active player, and `1` if the piece belongs to the passive player.

- `row` and `column`:

  - If the piece is on the board, then the `row` and `column` fields hold the row and column of the piece on the board, respectively (zero-based indexing).

    Row zero is defined as the active player's home row.

    Column zero is defined as the column where sente's elephant is located in the initial position.

  - If the piece is in the hand, then the `row` field and the `column` field are both `0b11`.

- `promotion`: The promotion bit is `1` if the piece is promoted, and `0` if the piece is not promoted.

## Action representation (7 bits total)

An action is represented by 7 bits.

| actor  | destinationRow | destinationColumn |
| ------ | -------------- | ----------------- |
| 3 bits | 2 bits         | 2 bits            |

- `actor`: The actor is the piece being moved or dropped.

  We use the following encoding:

  - `0b001` for activeLion
  - `0b010` for chick0
  - `0b011` for chick1
  - `0b100` for elephant0
  - `0b101` for elephant1
  - `0b110` for giraffe0
  - `0b111` for giraffe1

  The passive lion cannot move (by definition), so we do not assign an encoding to it.

- `destinationRow` and `destinationColumn`: Self-explanatory.

Note that action representation is **not** unique.
For example, if the active player has two chicks in hand, then dropping `chick0` in square `(0, 0)` and dropping `chick1` in the same square would have to distinct representations, even though they are the same action.
However, we have deemed this inefficiency to be acceptable.

The value `0b000_0000` represents a "null action". We use this when a states has no remaining actions to explore.

## Board representation (64 bits total)

A board is 12 squares. Each square is represented by 4 bits.
However, the squares in column 3 contain "don't care" bits.

| DONTCARE | r3c2 | r3c1 | r3c0 | DONTCARE | r2c2 | r2c1 | r2c0 | DONTCARE | r1c2 | r1c1 | r1c0 | DONTCARE | r0c2 | r0c1 | r0c0 |
| r3c3 | r3c2 | r3c1 | r3c0 | r2c3 | r2c2 | r2c1 | r2c0 | r1c3 | r1c2 | r1c1 | r1c0 | r0c3 | r0c2 | r0c1 | r0c0 |
| ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- |
| 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits |

The reason we store the "don't care" bits instead of simply omitting the squares in column 3.
This is to make it easy to calculate a bit offset from a set of coordinates.
If we omitted the squares in column 3, the code would be:

```rust
fn offset(coords: u8) -> u8 {
    let row = coords >> 2;
    let column = coords & 0b11;
    (row * 3 + column) * 4
}
```

But since we include the squares in column 3, the code can be simplified to:

```rust
fn offset(coords: u8) -> u8 {
    coords * 4
}
```

This code simplification comes at no additional memory cost.
This is because even if we omitted the squares in column 3, the size of the `Board` struct would only shrink to 48 bits, which still requires a 64-bit integer to store.

### Square representation (4 bits total)

| allegiance | piece  |
| ---------- | ------ |
| 1 bit      | 3 bits |

- `allegiance`: The allegiance bit is `0` if the piece belongs to the active player, and `1` if the piece belongs to the passive player.

  If the square is empty, the allegiance bit is zero.

- The `piece` bits represent the piece on the square.
  We use the following encoding:

  - `0b000` for an empty square
  - `0b001` for active lion and passive lion

  You can use the `allegiance` bit to determine which lion it is.

  - `0b010` for chick0
  - `0b011` for chick1
  - `0b100` for elephant0
  - `0b101` for elephant1
  - `0b110` for giraffe0
  - `0b111` for giraffe1

Observe that the `piece` encoding is similar to the `actor` encoding we use in the action representation.
The only difference is that `0b001` is used for both lions.

## Coordinate set representation (16 bits total)

When calculating which actions are legal in a given state,
we check whether a piece of a given species can legally move
from some start square to some destination square.
The species and destination square are known at compile time.
So, we implement this in a very straightforward way:
check to see if the start square is in a set of "legal start squares"
corresponding to the given species and destination square.

The question then becomes how to represent such a set.
Since square coordinates are represented with a 4-bit integer,
we can simply use a 16-bit integer as a bitset.

The bit at index `4 * row + column` is set if the square is in the set.

| ZERO  | r3c2  | r3c1  | r3c0  | ZERO  | r2c2  | r2c1  | r2c0  | ZERO  | r1c2  | r1c1  | r1c0  | ZERO  | r0c2  | r0c1  | r0c0  |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit | 1 bit |

Since there are only 3 columns, for any `n`, the bit for square `(row: n, column: 3)` is always zero.
