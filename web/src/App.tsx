import React from "react";
import * as imageUrls from "./images/urls";

const _256_POW_2 = 256 * 256;
const _256_POW_3 = 256 * 256 * 256;
const _256_POW_4 = 256 * 256 * 256 * 256;

const PACKETS_PER_DIRECTORY = 1000;

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

type SquareSelection = NoSelection | BoardSelection | ForestHandSelection;

interface NoSelection {
  readonly kind: SquareSelectionKind.None;
}

interface BoardSelection {
  readonly kind: SquareSelectionKind.Board;
  readonly squareIndex: number;
}

interface ForestHandSelection {
  readonly kind: SquareSelectionKind.Hand;
  readonly species: Species;
}

type Action = Move | Drop;

interface Move {
  readonly isDrop: false;
  readonly from: number;
  readonly to: number;
}

interface Drop {
  readonly isDrop: true;
  readonly species: Species;
  readonly to: number;
}

interface MutCache {
  paddedPacketMaximums: null | readonly number[];
  readonly packetMap: { [key: number]: Uint8Array };
}

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
        <div id="Board">
          {game.board.map((_, i) => this.renderBoardSquare(i))}
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
        className={`Square Square--i${squareIndex}${
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

  onBoardSquareClick(clickedSquareIndex: number): void {
    const prevSelection = this.state.squareSelection;
    const { game } = this.state;

    // Handle piece selection.
    if (
      prevSelection.kind === SquareSelectionKind.None &&
      clickedSquareIndex < game.board.length &&
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
        from: prevSelection.squareIndex,
        to: clickedSquareIndex,
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
    if (prevSelection.kind === SquareSelectionKind.Hand) {
      const action: Action = {
        isDrop: true,
        species: prevSelection.species,
        to: clickedSquareIndex,
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

function compressGameState(state: GameState): number {
  // TODO
  return 0;
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

function tryApplyAction(game: GameState, action: Action): null | GameState {
  // TODO
  return null;
}
