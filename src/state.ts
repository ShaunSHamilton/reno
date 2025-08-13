import { Dispatch, createContext } from "react";

type Data = {
  data: DataType;
  timestamp: number;
};

type DataType = {
  Levels?: Levels;
  CellVolts?: CellVolts;
  Temps?: Temps;
};

type Levels = {
  current: number;
  volt: number;
  charge_level: number;
  capacity: number;
};

type CellVolts = {
  cell_volts: number[];
};

type Temps = {
  temps: number[];
};

export const DataContext = createContext<Data[]>([]);
export const DataDispatchContext = createContext<Dispatch<any>>(() => {});

type DataAction = {
  type: "push";
  data: Data;
};

export function dataReducer(data: Data[], action: DataAction) {
  switch (action.type) {
    case "push": {
      return [...data, action.data];
    }
    default: {
      throw Error("Unknown action: " + action.type);
    }
  }
}
