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
  readonly board: Board;
  readonly activePlayer: Player;
}

type Board = readonly Square[];

type ActiveBoard = readonly ActiveSquare[];

interface Hand {
  readonly [Species.Bird]: number;
  readonly [Species.Elephant]: number;
  readonly [Species.Giraffe]: number;
  readonly [Species.Lion]: number;
}

type Square = EmptySquare | OccupiedSquare;

type ActiveSquare = EmptySquare | ActiveOccupiedSquare;

interface EmptySquare {
  readonly isEmpty: true;
}

interface OccupiedSquare {
  readonly isEmpty: false;
  readonly allegiance: Player;
  readonly species: Species;
  readonly isPromoted: boolean;
}

interface ActiveOccupiedSquare {
  readonly isEmpty: false;
  readonly isActive: boolean;
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

const _2_POW_4 = 2 ** 4;
const _2_POW_8 = 2 ** 8;
const _2_POW_13 = 2 ** 13;
const _2_POW_18 = 2 ** 18;
const _2_POW_23 = 2 ** 23;
const _2_POW_28 = 2 ** 28;
const _2_POW_34 = 2 ** 34;

const _256_POW_2 = 256 ** 2;
const _256_POW_3 = 256 ** 3;
const _256_POW_4 = 256 ** 4;

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

    if (isGameOver(game)) {
      return;
    }

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
    if (isGameOver(this.state.game)) {
      return;
    }

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

function compressGameState(game: GameState): number {
  const [activeHand, passiveHand] =
    game.activePlayer === Player.Forest
      ? [game.forestHand, game.skyHand]
      : [game.skyHand, game.forestHand];

  return compressActiveGameState(getActiveBoard(game), activeHand, passiveHand);
}

function getActiveBoard(game: GameState): ActiveBoard {
  if (game.activePlayer === Player.Forest) {
    return getActiveBoardAssumingForestIsActive(game.board);
  }

  return invertActiveBoard(getActiveBoardAssumingForestIsActive(game.board));
}

function getActiveBoardAssumingForestIsActive(board: Board): ActiveBoard {
  return board.map((square) => {
    if (square.isEmpty) {
      return { isEmpty: true };
    }

    return {
      isEmpty: false,
      isActive: square.allegiance === Player.Forest,
      species: square.species,
      isPromoted: square.isPromoted,
    };
  });
}

function invertActiveBoard(board: ActiveBoard): ActiveBoard {
  return board.map((_, squareIndex) => {
    const invertedIndex = invertSquareIndex(squareIndex);
    return invertActiveSquare(board[invertedIndex]);
  });
}

function invertSquareIndex(squareIndex: number): number {
  return 11 - squareIndex;
}

function invertActiveSquare(square: ActiveSquare): ActiveSquare {
  if (square.isEmpty) {
    return { isEmpty: true };
  }

  return {
    isEmpty: false,
    isActive: !square.isActive,
    species: square.species,
    isPromoted: square.isPromoted,
  };
}

function compressActiveGameState(
  board: ActiveBoard,
  activeHand: Hand,
  passiveHand: Hand
): number {
  const birds = [];
  const elephants = [];
  const giraffes = [];
  let activeLion = 0;
  let passiveLion = 0;

  for (let i = 0; i < 12; ++i) {
    const square = board[i];
    if (square.isEmpty) {
      continue;
    }

    const coords = getCoordsFromSquareIndex(i);

    switch (square.species) {
      case Species.Bird:
        birds.push(
          (Number(!square.isActive) << 5) |
            (coords << 1) |
            Number(square.isPromoted)
        );
        break;

      case Species.Elephant:
        elephants.push((Number(!square.isActive) << 4) | coords);
        break;

      case Species.Giraffe:
        giraffes.push((Number(!square.isActive) << 4) | coords);
        break;

      case Species.Lion:
        if (square.isActive) {
          activeLion = coords;
        } else {
          passiveLion = coords;
        }
        break;

      default:
        return typesafeUnreachable(square.species);
    }
  }

  for (const species of HAND_SPECIES) {
    for (let j = 0; j < activeHand[species]; ++j) {
      switch (species) {
        case Species.Bird:
          birds.push(0b011110);
          break;

        case Species.Elephant:
          elephants.push(0b01111);
          break;

        case Species.Giraffe:
          giraffes.push(0b01111);
          break;

        case Species.Lion:
          activeLion = 0b1111;
          break;

        default:
          return typesafeUnreachable(species);
      }
    }

    for (let j = 0; j < passiveHand[species]; ++j) {
      switch (species) {
        case Species.Bird:
          birds.push(0b111110);
          break;

        case Species.Elephant:
          elephants.push(0b11111);
          break;

        case Species.Giraffe:
          giraffes.push(0b11111);
          break;

        case Species.Lion:
          passiveLion = 0b1111;
          break;

        default:
          return typesafeUnreachable(species);
      }
    }
  }

  return buildCompressedGameState(
    birds,
    elephants,
    giraffes,
    activeLion,
    passiveLion
  );
}

function buildCompressedGameState(
  unsortedBirds: readonly number[],
  unsortedElephants: readonly number[],
  unsortedGiraffes: readonly number[],
  activeLion: number,
  passiveLion: number
): number {
  const [bird0, bird1] = unsortedBirds.slice().sort((a, b) => a - b);
  const [elephant0, elephant1] = unsortedElephants
    .slice()
    .sort((a, b) => a - b);
  const [giraffe0, giraffe1] = unsortedGiraffes.slice().sort((a, b) => a - b);
  return (
    passiveLion +
    _2_POW_4 * activeLion +
    _2_POW_8 * giraffe1 +
    _2_POW_13 * giraffe0 +
    _2_POW_18 * elephant1 +
    _2_POW_23 * elephant0 +
    _2_POW_28 * bird1 +
    _2_POW_34 * bird0
  );
}

function getCoordsFromSquareIndex(squareIndex: number): number {
  const row = Math.floor(squareIndex / 3);
  const col = squareIndex % 3;
  return (row << 2) | col;
}

function tryApplyAction(game: GameState, action: Action): null | GameState {
  const actions = getActions(game);
  if (!actions.some((other) => areActionsEqual(action, other))) {
    return null;
  }

  return unsafeApplyAction(game, action);
}

function getActions(game: GameState): readonly Action[] {
  const activeHand =
    game.activePlayer === Player.Forest ? game.forestHand : game.skyHand;

  if (isGameOver(game)) {
    return [];
  }

  const { board, activePlayer } = game;

  const out: Action[] = [];

  if (activeHand[Species.Bird] > 0) {
    writeDropForEachDest(game, Species.Bird, out);
  }

  if (activeHand[Species.Elephant] > 0) {
    writeDropForEachDest(game, Species.Elephant, out);
  }

  if (activeHand[Species.Giraffe] > 0) {
    writeDropForEachDest(game, Species.Giraffe, out);
  }

  for (let startIndex = 0; startIndex < 12; ++startIndex) {
    const actor = board[startIndex];
    if (!actor.isEmpty && actor.allegiance === activePlayer) {
      writeMoveForEachDest(
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

function isGameOver(game: GameState): boolean {
  return didPassivePlayerWin(game) || didActivePlayerWin(game);
}

function didPassivePlayerWin(game: GameState): boolean {
  const passiveHand =
    game.activePlayer === Player.Forest ? game.skyHand : game.forestHand;

  return passiveHand[Species.Lion] > 0;
}

function didActivePlayerWin(game: GameState): boolean {
  const { board, activePlayer } = game;
  // Check whether the active player won by the Try Rule.
  return activePlayer === Player.Forest
    ? isForestLion(board[9]) ||
        isForestLion(board[10]) ||
        isForestLion(board[11])
    : isSkyLion(board[0]) || isSkyLion(board[1]) || isSkyLion(board[2]);
}

function writeDropForEachDest(
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

function writeMoveForEachDest(
  game: GameState,
  startIndex: number,
  species: Species,
  isPromoted: boolean,
  out: Action[]
): void {
  const { activePlayer } = game;
  for (let destIndex = 0; destIndex < 12; ++destIndex) {
    const dest = game.board[destIndex];
    const isDestActive = !dest.isEmpty && dest.allegiance === activePlayer;
    if (
      !isDestActive &&
      canSpeciesMove(
        game.activePlayer,
        species,
        isPromoted,
        startIndex,
        destIndex
      )
    ) {
      out.push({ isDrop: false, startIndex: startIndex, destIndex: destIndex });
    }
  }
}

function canSpeciesMove(
  activePlayer: Player,
  species: Species,
  isPromoted: boolean,
  startIndex: number,
  destIndex: number
): boolean {
  const moveSet = getMoveSet(activePlayer, species, isPromoted);
  return doesMoveSetPermit(moveSet, startIndex, destIndex);
}

function getMoveSet(
  activePlayer: Player,
  species: Species,
  isPromoted: boolean
): MoveSet {
  const forestMoveSet = getForestMoveSet(species, isPromoted);

  return activePlayer === Player.Forest
    ? forestMoveSet
    : invertMoveSet(forestMoveSet);
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

function isSkyLion(square: Square): boolean {
  return (
    !square.isEmpty &&
    square.allegiance === Player.Sky &&
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

function unsafeApplyAction(game: GameState, action: Action): GameState {
  const out = cloneGameState(game);
  const { board } = out;
  const { activePlayer } = game;
  const activeHand =
    activePlayer === Player.Forest ? out.forestHand : out.skyHand;
  const captive = board[action.destIndex];

  if (action.isDrop) {
    board[action.destIndex] = {
      isEmpty: false,
      allegiance: activePlayer,
      species: action.species,
      isPromoted: false,
    };

    activeHand[action.species] -= 1;

    return invertActivePlayer(out);
  }

  if (!captive.isEmpty) {
    activeHand[captive.species] += 1;
  }

  const actor = board[action.startIndex];

  const inPromotionZone =
    game.activePlayer === Player.Forest
      ? action.destIndex > 8
      : action.destIndex < 3;

  const isActorBird = !actor.isEmpty && actor.species === Species.Bird;

  board[action.destIndex] =
    inPromotionZone && isActorBird ? { ...actor, isPromoted: true } : actor;

  board[action.startIndex] = { isEmpty: true };

  return invertActivePlayer(out);
}

function invertActivePlayer(game: GameState): GameState {
  return {
    ...game,
    activePlayer: invertPlayer(game.activePlayer),
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

function invertMoveSet(moveSet: MoveSet): MoveSet {
  return {
    n: moveSet.s,
    ne: moveSet.sw,
    e: moveSet.w,
    se: moveSet.nw,
    s: moveSet.n,
    sw: moveSet.ne,
    w: moveSet.e,
    nw: moveSet.se,
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
