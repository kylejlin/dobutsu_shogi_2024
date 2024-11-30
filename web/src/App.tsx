import React from "react";

const _256_POW_2 = 256 * 256;
const _256_POW_3 = 256 * 256 * 256;
const _256_POW_4 = 256 * 256 * 256 * 256;

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

interface Props {}

interface State {
  readonly game: GameState;
  readonly selectedSquareIndex: number | null;
  readonly cacheGeneration: number;
}

interface GameState {
  readonly forestHand: readonly Species[];
  readonly skyHand: readonly Species[];
  readonly board: readonly Square[];
  readonly activePlayer: Player;
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
  packetMaximums: null | readonly number[];
  readonly packetMap: { [key: number]: Uint8Array };
}

export class App extends React.Component<Props, State> {
  private readonly cache: MutCache;
  private readonly packetMaximumsPromise: Promise<readonly number[]>;
  private readonly resolvePacketMaximumsPromise: (
    value: readonly number[]
  ) => void;

  constructor(props: Props) {
    super(props);

    this.state = {
      game: getInitialGameState(),
      selectedSquareIndex: null,
      cacheGeneration: 0,
    };

    this.cache = {
      packetMaximums: null,
      packetMap: {},
    };

    this.resolvePacketMaximumsPromise = (): void => {
      throw new Error("resolvePacketMaximumsPromise not initialized");
    };

    this.packetMaximumsPromise = new Promise((resolve) => {
      (this as any).resolvePacketMaximumsPromise = resolve;
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

        if (this.cache.packetMaximums !== null) {
          // If the cache was already initialized, do nothing.
          return;
        }

        this.cache.packetMaximums = decodePacketMaximumBuffer(
          new Uint8Array(buffer)
        );
        this.resolvePacketMaximumsPromise(this.cache.packetMaximums);

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

    this.packetMaximumsPromise.then((packetMaximums) => {
      const url = getPacketUrl(compressedState, packetMaximums);
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
    return (
      <div id="App">
        <div id="Board">
          <div className="Square Square--i0"></div>
        </div>
      </div>
    );
  }
}

function getInitialGameState(): GameState {
  return {
    forestHand: [],
    skyHand: [],
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
  packetMaximums: readonly number[]
): string {
  // TODO
  return "";
}

function decodePacketMaximumBuffer(leBuffer: Uint8Array): readonly number[] {
  if (leBuffer.length % 5 !== 0) {
    throw new Error(
      `Expected buffer length to be a multiple of 5, but got a length of ${leBuffer.length}.`
    );
  }

  const maximums = [];

  for (let i = 0; i < leBuffer.length; i += 5) {
    maximums.push(
      leBuffer[i] +
        leBuffer[i + 1] * 256 +
        leBuffer[i + 2] * _256_POW_2 +
        leBuffer[i + 3] * _256_POW_3 +
        leBuffer[i + 4] * _256_POW_4
    );
  }

  return maximums;
}
