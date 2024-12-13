import React from "react";
import * as imageUrls from "./images/urls";

enum Species {
  Bird = "Bird",
  Elephant = "Elephant",
  Giraffe = "Giraffe",
  Lion = "Lion",
}

enum Player {
  Forest = "Forest",
  Sky = "Sky",
}

enum SquareSelectionKind {
  None = "None",
  Board = "Board",
  Hand = "Hand",
}

interface Props {}

interface State {
  readonly game: GameState;
  readonly squareSelection: SquareSelection;
  readonly cacheGeneration: number;
}

interface GameState {
  readonly forestHand: Hand;
  readonly skyHand: Hand;
  readonly board: readonly Square[];
  readonly activePlayer: Player;
}

interface Hand {
  readonly [Species.Bird]: number;
  readonly [Species.Elephant]: number;
  readonly [Species.Giraffe]: number;
  readonly [Species.Lion]: number;
}

type Square = EmptySquare | OccupiedSquare;

interface EmptySquare {
  readonly isEmpty: true;
}

interface OccupiedSquare {
  readonly isEmpty: false;
  readonly allegiance: Player;
  readonly species: Species;
  readonly isPromoted: boolean;
}

type SquareSelection = NoSelection | BoardSelection | HandSelection;

interface NoSelection {
  readonly kind: SquareSelectionKind.None;
}

interface BoardSelection {
  readonly kind: SquareSelectionKind.Board;
  readonly squareIndex: number;
}

interface HandSelection {
  readonly kind: SquareSelectionKind.Hand;
  readonly player: Player;
  readonly species: Species;
}

type Action = Move | Drop;

interface Move {
  readonly isDrop: false;
  readonly startIndex: number;
  readonly destIndex: number;
}

interface Drop {
  readonly isDrop: true;
  readonly species: Species;
  readonly destIndex: number;
}

interface MoveSet {
  readonly n: boolean;
  readonly ne: boolean;
  readonly e: boolean;
  readonly se: boolean;
  readonly s: boolean;
  readonly sw: boolean;
  readonly w: boolean;
  readonly nw: boolean;
}

interface MutCache {
  paddedPacketMaximums: null | readonly number[];
  readonly packetMap: { [key: number]: Uint8Array };
}

type Writable<T> = T extends object
  ? { -readonly [P in keyof T]: Writable<T[P]> }
  : T;

const _256_POW_2 = 256 * 256;
const _256_POW_3 = 256 * 256 * 256;
const _256_POW_4 = 256 * 256 * 256 * 256;

const PACKETS_PER_DIRECTORY = 1000;

const FOREST_CHICK_MOVE_SET = getMoveSetOf(["n"]);
const FOREST_HEN_MOVE_SET = getMoveSetUnion(
  getOrthogonalMoveSet(),
  getMoveSetOf(["ne", "nw"])
);
const FOREST_ELEPHANT_MOVE_SET = getDiagonalMoveSet();
const FOREST_GIRAFFE_MOVE_SET = getOrthogonalMoveSet();
const FOREST_LION_MOVE_SET = getMoveSetUnion(
  getOrthogonalMoveSet(),
  getDiagonalMoveSet()
);

const HAND_SPECIES = [
  Species.Bird,
  Species.Elephant,
  Species.Giraffe,
  Species.Lion,
] as const;

export class App extends React.Component<Props, State> {
  private readonly cache: MutCache;
  private readonly paddedPacketMaximumsPromise: Promise<readonly number[]>;
  private readonly resolvePaddedPacketMaximumsPromise: (
    paddedPacketMaximums: readonly number[]
  ) => void;

  constructor(props: Props) {
    super(props);

    this.state = {
      game: getInitialGameState(),
      squareSelection: { kind: SquareSelectionKind.None },
      cacheGeneration: 0,
    };

    this.cache = {
      paddedPacketMaximums: null,
      packetMap: {},
    };

    this.resolvePaddedPacketMaximumsPromise = (): void => {
      throw new Error("resolvePaddedPacketMaximumsPromise not initialized");
    };

    this.paddedPacketMaximumsPromise = new Promise((resolve) => {
      (this as any).resolvePaddedPacketMaximumsPromise = resolve;
    });
  }

  componentDidMount(): void {
    this.initializeCache();
  }

  initializeCache(): void {
    const maximumsUrl = getPacketMaximumsUrl();
    fetch(maximumsUrl)
      .then((response) => {
        if (!response.ok) {
          throw new Error(`Failed to fetch ${maximumsUrl}`);
        }
        return response.arrayBuffer();
      })
      .then((buffer) => {
        if (buffer.byteLength === 0) {
          throw new Error(`Received empty ArrayBuffer from ${maximumsUrl}`);
        }

        if (this.cache.paddedPacketMaximums !== null) {
          // If the cache was already initialized, do nothing.
          return;
        }

        this.cache.paddedPacketMaximums =
          decodePacketMaximumBufferAndPadWithInfinities(new Uint8Array(buffer));
        this.resolvePaddedPacketMaximumsPromise(
          this.cache.paddedPacketMaximums
        );

        this.incrementCacheGeneration();

        this.fetchPacketForCurrentGameStateIfNeeded();
      });
  }

  incrementCacheGeneration(): void {
    this.setState((prevState) => ({
      ...prevState,
      cacheGeneration: prevState.cacheGeneration + 1,
    }));
  }

  fetchPacketForCurrentGameStateIfNeeded(): void {
    this.fetchPacketForGameStateIfNeeded(this.state.game);
  }

  fetchPacketForGameStateIfNeeded(state: GameState): void {
    const compressedState = compressGameState(state);
    if (compressedState in this.cache.packetMap) {
      return;
    }

    this.paddedPacketMaximumsPromise.then((paddedPacketMaximums) => {
      const url = getPacketUrl(compressedState, paddedPacketMaximums);
      fetch(url)
        .then((response) => {
          if (!response.ok) {
            throw new Error(`Failed to fetch ${url}`);
          }
          return response.arrayBuffer();
        })
        .then((buffer) => {
          if (buffer.byteLength === 0) {
            throw new Error(`Received empty ArrayBuffer from ${url}`);
          }

          this.cache.packetMap[compressedState] = new Uint8Array(buffer);
          this.incrementCacheGeneration();
        });
    });
  }

  render(): React.ReactElement {
    const { game } = this.state;
    return (
      <div id="App">
        <div id="SkyHand">
          {HAND_SPECIES.map((species) =>
            this.renderHandSquare(Player.Sky, species, game.skyHand[species])
          )}
        </div>

        <div id="Board">
          {game.board.map((_, i) => this.renderBoardSquare(i))}
        </div>

        <div id="ForestHand">
          {HAND_SPECIES.map((species) =>
            this.renderHandSquare(
              Player.Forest,
              species,
              game.forestHand[species]
            )
          )}
        </div>
      </div>
    );
  }

  renderBoardSquare(squareIndex: number): React.ReactElement {
    const { game, squareSelection } = this.state;
    const selectedBoardSquareIndex =
      squareSelection.kind === SquareSelectionKind.Board
        ? squareSelection.squareIndex
        : null;
    return (
      <div
        className={`Square Square--board${squareIndex}${
          selectedBoardSquareIndex === squareIndex ? " Square--selected" : ""
        }`}
        key={squareIndex}
      >
        <img
          alt={getSquareAltText(game.board[squareIndex])}
          src={getSquareImageSrc(game.board[squareIndex])}
          onClick={(): void => this.onBoardSquareClick(squareIndex)}
        />
      </div>
    );
  }

  renderHandSquare(
    player: Player,
    species: Species,
    count: number
  ): React.ReactElement {
    const speciesIndex = HAND_SPECIES.indexOf(species);
    if (speciesIndex === -1) {
      throw new Error(`Invalid species: ${species}`);
    }

    const { game, squareSelection } = this.state;
    const selectedFriendlyHandSpecies =
      squareSelection.kind === SquareSelectionKind.Hand &&
      squareSelection.player === player
        ? squareSelection.species
        : null;
    const handSquare: Square =
      count === 0
        ? { isEmpty: true }
        : {
            isEmpty: false,
            allegiance: player,
            species,
            isPromoted: false,
          };
    return (
      <div
        className={`Square Square--hand${speciesIndex}${
          selectedFriendlyHandSpecies === species ? " Square--selected" : ""
        }`}
        key={speciesIndex}
      >
        <img
          alt={getSquareAltText(handSquare)}
          src={getSquareImageSrc(handSquare)}
          onClick={(): void => this.onHandSquareClick(player, species)}
        />
      </div>
    );
  }

  onBoardSquareClick(clickedSquareIndex: number): void {
    const prevSelection = this.state.squareSelection;
    const { game } = this.state;

    // Handle piece selection.
    if (
      prevSelection.kind === SquareSelectionKind.None &&
      (game.activePlayer === Player.Forest
        ? isSquareForest(game.board[clickedSquareIndex])
        : isSquareSky(game.board[clickedSquareIndex]))
    ) {
      this.setState({
        squareSelection: {
          kind: SquareSelectionKind.Board,
          squareIndex: clickedSquareIndex,
        },
      });
      return;
    }

    // Handle piece deselection.
    if (
      prevSelection.kind === SquareSelectionKind.Board &&
      prevSelection.squareIndex === clickedSquareIndex
    ) {
      this.setState({ squareSelection: { kind: SquareSelectionKind.None } });
      return;
    }

    // Handle move.
    if (prevSelection.kind === SquareSelectionKind.Board) {
      const action: Action = {
        isDrop: false,
        startIndex: prevSelection.squareIndex,
        destIndex: clickedSquareIndex,
      };
      const newGameState = tryApplyAction(game, action);
      if (newGameState !== null) {
        this.setState({
          game: newGameState,
          squareSelection: { kind: SquareSelectionKind.None },
        });
        this.fetchPacketForGameStateIfNeeded(newGameState);
      }
      return;
    }

    // Handle drop.
    if (
      prevSelection.kind === SquareSelectionKind.Hand &&
      prevSelection.player === game.activePlayer
    ) {
      const action: Action = {
        isDrop: true,
        species: prevSelection.species,
        destIndex: clickedSquareIndex,
      };
      const newGameState = tryApplyAction(game, action);
      if (newGameState !== null) {
        this.setState({
          game: newGameState,
          squareSelection: { kind: SquareSelectionKind.None },
        });
        this.fetchPacketForGameStateIfNeeded(newGameState);
      }
      return;
    }
  }

  onHandSquareClick(player: Player, species: Species): void {
    const prevSelection = this.state.squareSelection;

    // Handle piece selection.
    if (
      prevSelection.kind === SquareSelectionKind.None &&
      player === this.state.game.activePlayer
    ) {
      this.setState({
        squareSelection: {
          kind: SquareSelectionKind.Hand,
          player,
          species,
        },
      });
      return;
    }

    // Handle piece deselection.
    if (
      prevSelection.kind === SquareSelectionKind.Hand &&
      prevSelection.player === player &&
      prevSelection.species === species
    ) {
      this.setState({ squareSelection: { kind: SquareSelectionKind.None } });
      return;
    }
  }
}

function getInitialGameState(): GameState {
  return {
    forestHand: {
      [Species.Bird]: 0,
      [Species.Elephant]: 0,
      [Species.Giraffe]: 0,
      [Species.Lion]: 0,
    },
    skyHand: {
      [Species.Bird]: 0,
      [Species.Elephant]: 0,
      [Species.Giraffe]: 0,
      [Species.Lion]: 0,
    },
    board: [
      // row0
      {
        isEmpty: false,
        allegiance: Player.Forest,
        species: Species.Elephant,
        isPromoted: false,
      },
      {
        isEmpty: false,
        allegiance: Player.Forest,
        species: Species.Lion,
        isPromoted: false,
      },
      {
        isEmpty: false,
        allegiance: Player.Forest,
        species: Species.Giraffe,
        isPromoted: false,
      },
      // row1
      { isEmpty: true },
      {
        isEmpty: false,
        allegiance: Player.Forest,
        species: Species.Bird,
        isPromoted: false,
      },
      { isEmpty: true },
      // row2
      { isEmpty: true },
      {
        isEmpty: false,
        allegiance: Player.Sky,
        species: Species.Bird,
        isPromoted: false,
      },
      { isEmpty: true },
      // row3
      {
        isEmpty: false,
        allegiance: Player.Sky,
        species: Species.Giraffe,
        isPromoted: false,
      },
      {
        isEmpty: false,
        allegiance: Player.Sky,
        species: Species.Lion,
        isPromoted: false,
      },
      {
        isEmpty: false,
        allegiance: Player.Sky,
        species: Species.Elephant,
        isPromoted: false,
      },
    ],
    activePlayer: Player.Forest,
  };
}

function getPacketMaximumsUrl(): string {
  return "https://github.com/kylejlin/dobutsu_shogi_database_2024/raw/refs/heads/main/maximums.dat";
}

function getPacketUrl(
  compressedState: number,
  paddedPacketMaximums: readonly number[]
): string {
  const i = findPaddedPacketIndex(compressedState, paddedPacketMaximums);
  if (i === -1 || i === 0 || i === paddedPacketMaximums.length - 1) {
    throw new Error(
      `Failed to find padded packet index for compressed state ${compressedState}.`
    );
  }

  const j = i - 1;
  return `https://github.com/kylejlin/dobutsu_shogi_database_2024/raw/refs/heads/main/${String(
    Math.floor(j / PACKETS_PER_DIRECTORY)
  )}/${String(j % PACKETS_PER_DIRECTORY)}.dat`;
}

/**
 * @param paddedPacketMaximums This must be sorted in ascending order,
 * and must start and end with -Infinity and Infinity, respectively.
 */
function findPaddedPacketIndex(
  compressedState: number,
  paddedPacketMaximums: readonly number[]
): number {
  let inclusiveLow = 1;
  let inclusiveHigh = paddedPacketMaximums.length - 1;

  while (inclusiveLow <= inclusiveHigh) {
    const mid = Math.floor((inclusiveLow + inclusiveHigh) / 2);
    if (compressedState <= paddedPacketMaximums[mid - 1]) {
      inclusiveHigh = mid - 1;
    } else if (compressedState > paddedPacketMaximums[mid]) {
      inclusiveLow = mid + 1;
    } else {
      return mid;
    }
  }

  return -1;
}

function decodePacketMaximumBufferAndPadWithInfinities(
  leBuffer: Uint8Array
): readonly number[] {
  if (leBuffer.length % 5 !== 0) {
    throw new Error(
      `Expected buffer length to be a multiple of 5, but got a length of ${leBuffer.length}.`
    );
  }

  const maximums = [-Infinity];

  for (let i = 0; i < leBuffer.length; i += 5) {
    maximums.push(
      leBuffer[i] +
        leBuffer[i + 1] * 256 +
        leBuffer[i + 2] * _256_POW_2 +
        leBuffer[i + 3] * _256_POW_3 +
        leBuffer[i + 4] * _256_POW_4
    );
  }

  maximums.push(Infinity);

  return maximums;
}

function getSquareAltText(square: Square): string {
  if (square.isEmpty) {
    return "Empty Square";
  }

  return `${square.allegiance} ${square.species}${
    square.isPromoted ? " (Promoted)" : ""
  }`;
}

function getSquareImageSrc(square: Square): string {
  if (square.isEmpty) {
    return imageUrls.emptySquare;
  }

  switch (square.allegiance) {
    case Player.Sky:
      return getSkyImageSrc(square);

    case Player.Forest:
      return getForestImageSrc(square);

    default:
      return typesafeUnreachable(square.allegiance);
  }
}

function getSkyImageSrc(square: OccupiedSquare): string {
  const { species } = square;
  switch (species) {
    case Species.Bird:
      return square.isPromoted ? imageUrls.skyHen : imageUrls.skyChick;

    case Species.Elephant:
      return imageUrls.skyElephant;

    case Species.Giraffe:
      return imageUrls.skyGiraffe;

    case Species.Lion:
      return imageUrls.skyLion;

    default:
      return typesafeUnreachable(species);
  }
}

function getForestImageSrc(square: OccupiedSquare): string {
  const { species } = square;
  switch (species) {
    case Species.Bird:
      return square.isPromoted ? imageUrls.forestHen : imageUrls.forestChick;

    case Species.Elephant:
      return imageUrls.forestElephant;

    case Species.Giraffe:
      return imageUrls.forestGiraffe;

    case Species.Lion:
      return imageUrls.forestLion;

    default:
      return typesafeUnreachable(species);
  }
}

function typesafeUnreachable(impossible: never): never {
  return impossible;
}

function isSquareForest(square: Square): boolean {
  return !square.isEmpty && square.allegiance === Player.Forest;
}

function isSquareSky(square: Square): boolean {
  return !square.isEmpty && square.allegiance === Player.Sky;
}

function compressGameState(state: GameState): number {
  // TODO
  return 0;
}

function tryApplyAction(game: GameState, action: Action): null | GameState {
  if (game.activePlayer === Player.Forest) {
    return tryApplyActionForest(game, action);
  }

  const invertedResult = tryApplyActionForest(
    invertGameState(game),
    invertAction(action)
  );
  return invertedResult === null ? null : invertGameState(invertedResult);
}

function tryApplyActionForest(
  game: GameState,
  action: Action
): null | GameState {
  const actions = getForestActions(game);
  if (!actions.some((other) => areActionsEqual(action, other))) {
    return null;
  }

  return unsafeApplyForestAction(game, action);
}

function getForestActions(game: GameState): readonly Action[] {
  if (game.activePlayer !== Player.Forest) {
    return [];
  }

  // If the active lion is in the passive hand,
  // the passive player has won.
  if (game.skyHand[Species.Lion] > 0) {
    return [];
  }

  const { board, forestHand } = game;

  // If the active lion is in the passive home row,
  // the active player has won by the Try Rule.
  if (
    isForestLion(board[9]) ||
    isForestLion(board[10]) ||
    isForestLion(board[11])
  ) {
    return [];
  }

  const out: Action[] = [];

  if (forestHand[Species.Bird] > 0) {
    writeForestDropForEachDest(game, Species.Bird, out);
  }

  if (forestHand[Species.Elephant] > 0) {
    writeForestDropForEachDest(game, Species.Elephant, out);
  }

  if (forestHand[Species.Giraffe] > 0) {
    writeForestDropForEachDest(game, Species.Giraffe, out);
  }

  for (let startIndex = 0; startIndex < 12; ++startIndex) {
    const actor = board[startIndex];
    if (!actor.isEmpty && actor.allegiance === Player.Forest) {
      writeForestMoveForEachDest(
        game,
        startIndex,
        actor.species,
        actor.isPromoted,
        out
      );
    }
  }

  return out;
}

function writeForestDropForEachDest(
  game: GameState,
  species: Species,
  out: Action[]
): void {
  for (let destIndex = 0; destIndex < 12; ++destIndex) {
    if (game.board[destIndex].isEmpty) {
      out.push({ isDrop: true, species, destIndex: destIndex });
    }
  }
}

function writeForestMoveForEachDest(
  game: GameState,
  startIndex: number,
  species: Species,
  isPromoted: boolean,
  out: Action[]
): void {
  for (let destIndex = 0; destIndex < 12; ++destIndex) {
    const dest = game.board[destIndex];
    const isDestForest = !dest.isEmpty && dest.allegiance === Player.Forest;
    if (
      !isDestForest &&
      canForestSpeciesMove(species, isPromoted, startIndex, destIndex)
    ) {
      out.push({ isDrop: false, startIndex: startIndex, destIndex: destIndex });
    }
  }
}

function canForestSpeciesMove(
  species: Species,
  isPromoted: boolean,
  startIndex: number,
  destIndex: number
) {
  const moveSet = getForestMoveSet(species, isPromoted);
  return doesMoveSetPermit(moveSet, startIndex, destIndex);
}

function getForestMoveSet(species: Species, isPromoted: boolean): MoveSet {
  switch (species) {
    case Species.Bird:
      return isPromoted ? FOREST_HEN_MOVE_SET : FOREST_CHICK_MOVE_SET;

    case Species.Elephant:
      return FOREST_ELEPHANT_MOVE_SET;

    case Species.Giraffe:
      return FOREST_GIRAFFE_MOVE_SET;

    case Species.Lion:
      return FOREST_LION_MOVE_SET;

    default:
      return typesafeUnreachable(species);
  }
}

function doesMoveSetPermit(
  moveSet: MoveSet,
  startIndex: number,
  destIndex: number
): boolean {
  const startRow = Math.floor(startIndex / 3);
  const startCol = startIndex % 3;
  const destRow = Math.floor(destIndex / 3);
  const destCol = destIndex % 3;
  const dx = destCol - startCol;
  const dy = destRow - startRow;

  return (
    (moveSet.n && dx === 0 && dy === 1) ||
    (moveSet.ne && dx === 1 && dy === 1) ||
    (moveSet.e && dx === 1 && dy === 0) ||
    (moveSet.se && dx === 1 && dy === -1) ||
    (moveSet.s && dx === 0 && dy === -1) ||
    (moveSet.sw && dx === -1 && dy === -1) ||
    (moveSet.w && dx === -1 && dy === 0) ||
    (moveSet.nw && dx === -1 && dy === 1)
  );
}

function isForestLion(square: Square): boolean {
  return (
    !square.isEmpty &&
    square.allegiance === Player.Forest &&
    square.species === Species.Lion
  );
}

function areActionsEqual(action1: Action, action2: Action): boolean {
  if (action1.isDrop) {
    return (
      action2.isDrop &&
      action1.species === action2.species &&
      action1.destIndex === action2.destIndex
    );
  }

  return (
    !action2.isDrop &&
    action1.startIndex === action2.startIndex &&
    action1.destIndex === action2.destIndex
  );
}

function unsafeApplyForestAction(game: GameState, action: Action): GameState {
  if (game.activePlayer !== Player.Forest) {
    throw new Error("Cannot apply a Forest action when it is Sky's turn.");
  }

  const out = cloneGameState(game);
  const { board, forestHand } = out;
  const captive = board[action.destIndex];

  // TODO
  return game;
}

function invertGameState(game: GameState): GameState {
  return {
    forestHand: game.skyHand,
    skyHand: game.forestHand,
    board: game.board.map((_, i) => {
      const invertedIndex = invertBoardIndex(i);
      return invertSquareAllegiance(game.board[invertedIndex]);
    }),
    activePlayer: invertPlayer(game.activePlayer),
  };
}

function invertBoardIndex(index: number): number {
  const row = Math.floor(index / 3);
  const col = index % 3;
  const invertedRow = 3 - row;
  const invertedCol = 2 - col;
  return invertedRow * 3 + invertedCol;
}

function invertSquareAllegiance(square: Square): Square {
  if (square.isEmpty) {
    return square;
  }

  return {
    ...square,
    allegiance: invertPlayer(square.allegiance),
  };
}

function invertPlayer(player: Player): Player {
  switch (player) {
    case Player.Forest:
      return Player.Sky;

    case Player.Sky:
      return Player.Forest;

    default:
      return typesafeUnreachable(player);
  }
}

function invertAction(action: Action): Action {
  if (action.isDrop) {
    return {
      isDrop: action.isDrop,
      species: action.species,
      destIndex: invertBoardIndex(action.destIndex),
    };
  }

  return {
    isDrop: action.isDrop,
    startIndex: invertBoardIndex(action.startIndex),
    destIndex: invertBoardIndex(action.destIndex),
  };
}

function getMoveSetOf(directions: readonly (keyof MoveSet)[]): MoveSet {
  const out = {
    n: false,
    ne: false,
    e: false,
    se: false,
    s: false,
    sw: false,
    w: false,
    nw: false,
  };

  for (const direction of directions) {
    out[direction] = true;
  }

  return out;
}

function getOrthogonalMoveSet(): MoveSet {
  return getMoveSetOf(["n", "e", "s", "w"]);
}

function getDiagonalMoveSet(): MoveSet {
  return getMoveSetOf(["ne", "se", "sw", "nw"]);
}

function getMoveSetUnion(a: MoveSet, b: MoveSet): MoveSet {
  return {
    n: a.n || b.n,
    ne: a.ne || b.ne,
    e: a.e || b.e,
    se: a.se || b.se,
    s: a.s || b.s,
    sw: a.sw || b.sw,
    w: a.w || b.w,
    nw: a.nw || b.nw,
  };
}

function cloneGameState(game: GameState): Writable<GameState> {
  return {
    forestHand: { ...game.forestHand },
    skyHand: { ...game.skyHand },
    board: game.board.map((square) => ({ ...square })),
    activePlayer: game.activePlayer,
  };
}
