import { useReducer, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./app.css";
import { BluetoothAdapterControls } from "./bluetooth-adapter-controls";
import { PeripheralControls } from "./peripheral-controls";
import { Graphs } from "./graphs";
import { GraphControls } from "./graph-controls";
import { Settings } from "./settings";
import { DataContext, DataDispatchContext, dataReducer } from "./state";

export function App() {
  const [data, dispatch] = useReducer(dataReducer, []);

  return (
    <DataContext.Provider value={data}>
      <DataDispatchContext.Provider value={dispatch}>
        <div className="container">
          <h1>Reno</h1>
          <Settings />
          <form onSubmit={(e) => e.preventDefault()}>
            <BluetoothAdapterControls />
            <PeripheralControls />
            <GraphControls />
          </form>
          <Graphs />
        </div>
      </DataDispatchContext.Provider>
    </DataContext.Provider>
  );
}
