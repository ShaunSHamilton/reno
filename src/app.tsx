import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./app.css";
import { BluetoothAdapterControls } from "./bluetooth-adapter-controls";
import { PeripheralControls } from "./peripheral-controls";
import { Graphs } from "./graphs";
import { GraphControls } from "./graph-controls";

export function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <div className="container">
      <h1>Reno</h1>
      <form onSubmit={(e) => e.preventDefault()}>
        <BluetoothAdapterControls />
        <PeripheralControls />
        <GraphControls />
      </form>
      <Graphs />
    </div>
  );
}
