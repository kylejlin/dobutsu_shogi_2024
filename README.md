# Dobutsu Shogi Analyzer (2024)

This program solves Dobutsu Shogi.
The name is qualified with "2024" to distinguish it from
a similar solver I wrote in 2021.
I'm creating a new solver because I want to try a different approach.

## Table of contents

1. [Supremacy clause](#supremacy-clause)
2. [Rule simplifications](#rule-simplifications)
   1. [The Try Rule](#the-try-rule)
   2. [Threefold Repetition Rule](#threefold-repetition-rule)
3. [Definition of optimal play](#definition-of-optimal-play)
4. [State representation](#state-representation)
5. [Action representation](#action-representation)
6. [Search node representation](#search-node-representation)
7. [Board representation](#board-representation)

## Supremacy clause

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

## State representation (48 bits total)

A given game state is represented by 48 bits,
which stores the _timeless state_ and the number of plies played.

The _timeless state_ is a 40-bit integer that stores the positions and allegiances of the 8 pieces.
Normally, one might use the term "board" instead, especially in
chess contexts.
However, the "board" does not capture all the piece information
in Dobutsu Shogi, since pieces can also be in the _hand_.
So, we use the term "timeless state" to collectively refer to
both the board and the players' hands.

### State format (48 bits total)

| timelessState | plyCount |
| ------------- | -------- |
| 40 bits       | 8 bits   |

### Timeless state format (40 bits total)

The timeless state is a 40-bit integer that stores the positions and allegiances of the 8 pieces.
The 2 lions do not have an allegiance stored, since they do not change allegiance (because if they are captured, the game is over).
The 2 chicks have an additional bit to store their promotion status.

| chick0 | chick1 | elephant0 | elephant1 | giraffe0 | giraffe1 | lionActive | lionPassive |
| ------ | ------ | --------- | --------- | -------- | -------- | ---------- | ----------- |
| 6 bits | 6 bits | 5 bits    | 5 bits    | 5 bits   | 5 bits   | 4 bit      | 4 bit       |

Most significant bits are on the left.

### Chick state format (6 bits total)

| allegiance | row    | column | promotion |
| ---------- | ------ | ------ | --------- |
| 1 bit      | 2 bits | 2 bits | 1 bit     |

The allegiance bit is `0` if the chick belongs to the active player, and `1` if the chick belongs to the passive player. This convention is used for all pieces.

If the chick is on the board, then the row and column fields hold the row and column of the chick on the board, respectively (zero-based indexing).
Row zero is defined as the active player's home row.
For columns, the direction of counting doesn't matter
due to horizontal symmetry.
If the chick is in the hand, the both the row field and the column field are `0b11`.
This convention is used for all pieces.

The promotion bit is `1` if the chick is promoted, and `0` if the chick is not promoted.

There is an added requirement that `chick0 <= chick1`,
when `chick0` and `chick1` are treated as unsigned 6-bit integers. The same requirement holds for elephants and giraffes.

This requirement ensures that the each state has a unique representation.
If we didn't have this requirement, then we could have two representations that are identical except for the order of the pieces (e.g., one representation where `chick0` is `0b000000` and `chick1` is `0b101010`, and another representation where `chick0` is `0b101010` and `chick1` is `0b000000`), which would both map to the same state.

### Elephant and giraffe state format (5 bits total)

| allegiance | row    | column |
| ---------- | ------ | ------ |
| 1 bit      | 2 bits | 2 bits |

### Lion state format (4 bits total)

| row    | column |
| ------ | ------ |
| 2 bits | 2 bits |

### Ply count (8 bits total)

The ply count holds the number of plies that have been played so far.

## Action representation

An action is represented by 7 bits.

| piece  | destination row | destination column |
| ------ | --------------- | ------------------ |
| 3 bits | 2 bits          | 2 bits             |

The piece encoding is as follows:

- `0b001` for activeLion
- `0b010` for chick0
- `0b011` for chick1
- `0b100` for elephant0
- `0b101` for elephant1
- `0b110` for giraffe0
- `0b111` for giraffe1

The passive lion cannot move (by definition), so we do not assign an encoding to it.

Note that action representation is **not** unique.
For example, if the active player has two chicks in hand, then dropping `chick0` in square `(0, 0)` and dropping `chick1` in the same square would have to distinct representations, even though they are the same action.
However, we have deemed this inefficiency to be acceptable.

## Search node representation (64 bits total)

A search node is represented by 64 bits.

| state   | nextAction | bestDiscoveredOutcome |
| ------- | ---------- | --------------------- |
| 48 bits | 7 bits     | 9 bits                |

### State

We described the state format [above](#state-representation).

### Next action

The `nextAction` field is a 7-bit unsigned integer. For non-terminal nodes, it is initialized to `0b001_0000`.
When all legal actions have been explored, it is set to `0b000_0000`.
For terminal nodes, there are no legal actions to begin with, so the field is immediately initialized as `0b000_0000`.

### Best discovered outcome

The `bestDiscoveredOutcome` field is a 9-bit signed integer in two's complement format.

- If the value is zero, it means the best discovered outcome is a draw.
- If the value is `n` for `n > 0`, it means the best discovered outcome is a forced win for the active player in `201 - n` plies from the current state.
- If the value is `n` for `n < 0`, it means the best discovered outcome is a forced win for the passive player in `201 + n` plies from the current state.

"Best" is relative to the active player.

## Solution representation (64 bits total)

| timelessState | dontCare | bestOutcome |
| ------------- | -------- | ----------- |
| 40 bits       | 15 bits  | 9 bits      |

## Board representation (60 bits total)

| r3c2   | r3c1   | r3c0   | r2c2   | r2c1   | r2c0   | r1c2   | r1c1   | r1c0   | r0c2   | r0c1   | r0c0   |
| ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ | ------ |
| 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits | 5 bits |

### Square format (5 bits total)

| allegiance | pieceNumber | species |
| ---------- | ----------- | ------- |
| 1 bit      | 1 bits      | 3 bits  |

The allegiance bit is `0` if the piece belongs to the active player, and `1` if the piece belongs to the passive player.

For chicks, the piece number is `0` if the chick is `chick0`, and `1` if the chick is `chick1`. The same goes for elephants and giraffes.
For lions, the piece number is `0` if the lion is `activeLion` and `1` if the lion is `passiveLion`.
Note that the lion piece number bit is redundant, since it identical to the allegiance bit.

The species bits are as follows:

- `0b000` for an empty square
- `0b001` for a chick
- `0b010` for an elephant
- `0b011` for a giraffe
- `0b100` for a lion
