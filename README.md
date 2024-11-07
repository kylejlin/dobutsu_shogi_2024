# Dobutsu Shogi Analyzer (2024)

This program solves Dobutsu Shogi.
The name is qualified with "2024" to distinguish it from
a similar solver I wrote in 2021.
I'm creating a new solver because I want to try a different approach.

## Table of contents

1. [Official rules](#official-rules)
2. [Rule simplifications](#rule-simplifications)
   1. [The Try Rule](#the-try-rule)
   2. [Threefold Repetition Rule](#threefold-repetition-rule)
3. [Definition of optimal play](#definition-of-optimal-play)
4. [Algorithm](#algorithm)
5. [Forward node representation](#forward-node-representation)
6. [State representation](#state-representation)
7. [Timeless state representation](#timeless-state-representation-40-bits-total)
8. [Action representation](#action-representation)
9. [Board representation](#board-representation)
10. [Square set representation](#square-set-representation)
11. [Backward node representation](#backward-node-representation)

## Official rules

The term "official rules" refers to the [original Japanese rules](https://nekomado.com/wp/wp-content/uploads/2021/12/jp.pdf).

An English translation may be found [here](https://nekomado.com/wp/wp-content/uploads/2021/12/en.pdf).

In the event that there is a discrepancy between the Japanese rules and the English translation, the Japanese rules are considered authoritative. No known discrepancies exist at the time of writing.

In case the documents become inaccessible, PDFs can be found in the [`docs`](./docs) directory of this repository.

## Rule simplifications

However, we make some simplifications to the rules
to make it easier to implement our tree search algorithm.

**These simplifications preserve the correctness and optimality of winning and losing strategies.**
That is, a player has a strategy that guarantees a win in N moves under the official rules if and only if they have a strategy that guarantees a win in N moves under the simplified rules. The same goes for losing strategies. I might later prove this formally.

Note that the simplifications do not preserve the correctness of draw analyses, meaning that even if a given position in the simplified rules is a draw, it is not necessarily a draw in the official rules.
In practice, this is not a significant issue,
since solver concluded that the initial position is
winning for 後手.

### 1. The Try Rule

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

### 2. Threefold Repetition Rule

We remove the threefold repetition rule.

Instead, we place an arbitrary _N_ ply limit on the game.
That is, at the end of the N-th ply, if neither player has won, the game is a draw.
For the purpose of this project, we set _N_ to 200.

A ply is a single move by a single player.
We use the term "ply" instead of "move" to avoid
confusion that often arises in the chess world,
where "move" can refer to a _pair_ of moves by both players.

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

## Forward node representation (64 bits total)

Forward nodes are used during the first step of the algorithm (i.e., calculating the set of all reachable states).

| state   | nextAction | ZERO   |
| ------- | ---------- | ------ |
| 48 bits | 7 bits     | 9 bits |

- `state`: see [State representation](#state-representation)
- `nextAction`: see [Action representation](#action-representation).

  If there are no remaining actions to explore, then this is zero.

  When a node is newly created,
  we initialize `nextAction` to `0b001_0000` if the node state is non-terminal, and `0` if the node state is terminal.

- `ZERO`: These bits are unused, so we set them to zero.

## State representation (48 bits total)

| timelessState | plyCount |
| ------------- | -------- |
| 40 bits       | 8 bits   |

- `timelessState`: See [Timeless state format](#timeless-state-format-40-bits-total). This stores the positions, allegiances, and promotion statuses of the pieces.
- `plyCount`: The number of plies that have been played so far,
  encoded as an 8-bit unsigned integer.

## Timeless state representation (40 bits total)

The timeless state is a 40-bit integer that stores the positions and allegiances, and promotion statuses of the 8 pieces.
Most pieces require 5 bits (1 for allegiance, 4 for position).
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

## Action representation

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

## Board representation (48 bits total)

A board is 12 squares. Each square is represented by 4 bits.

| r3c2   | r3c1   | r3c0   | r2c2   | r2c1   | r2c0   | r1c2   | r1c1   | r1c0   | r0c2   | r0c1   | r0c0   |
| ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ |
| 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits | 4 bits |

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

## Square set representation (16 bits total)

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

## Backward node representation (64 bits total)

Backward nodes are used during the first step of the algorithm (i.e., retrograde analysis).

| state   | unknownChildCount | bestKnownOutcome |
| ------- | ----------------- | ---------------- |
| 48 bits | 7 bits            | 9 bits           |

Observe that the format is very similar to [that of forward nodes](#forward-node-representation-64-bits-total). The only differences are:

1. Instead of storing the `nextAction`, we store the `unknownChildCount`. Every time a node with a known best outcome is visited, we decrement the `unknownChildCount` of its parent. When the `unknownChildCount` reaches zero, the parent's best outcome is now known.
2. We also store the `bestKnownOutcome`.
   This is a two's complement 9-bit signed integer that represents the best known outcome of the state.

   - `0` represents a draw.
   - A positive number `n` represents a win for the active player
     in `201 - n` plies.
   - A negative number `-n` represents a win for the passive player
     in `201 + n` plies.
