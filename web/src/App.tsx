import React from "react";

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

export class App extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      game: getInitialGameState(),
    };
  }

  render(): React.ReactElement {
    return <div className="App">Hello world</div>;
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
