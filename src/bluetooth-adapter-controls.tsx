import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

export function BluetoothAdapterControls() {
  const [status, setStatus] = useState("Not Connected");
  const [adapter, setAdapter] = useState("");
  const [adapters, setAdapters] = useState<string[]>([]);
  async function refreshAdapters() {
    setStatus("Refreshing Adapters");
    try {
      const ads: string[] = await invoke("refresh_bluetooth_adapters");
      setStatus(`Found ${ads.length} Adapters`);
      setAdapters(ads);
    } catch (e) {
      console.error(e);
      setStatus("Error Refreshing Adapters");
    }
  }
  return (
    <fieldset>
      <legend>Bluetooth Adapter</legend>
      <select
        value={adapter}
        onChange={(e) => {
          console.log(e);
          setAdapter(e.target.value);
        }}
      >
        <option value="">Select an Adapter</option>
        {adapters.map((adapter) => (
          <option key={adapter} value={adapter}>
            {adapter}
          </option>
        ))}
      </select>
      <button
        onClick={() => {
          refreshAdapters();
        }}
      >
        Refresh Adapters
      </button>
      <button
        onClick={() => {
          console.log(adapter, adapters);
          setStatus("Connecting to Adapter");
          if (adapter === "") {
            setStatus("No Adapter Selected");
            return;
          }
          invoke("connect_to_adapter", { name: adapter })
            .then(() => {
              setStatus("Connected to Adapter");
            })
            .catch((e) => {
              console.error(e);
              setStatus("Error Connecting to Adapter");
            });
        }}
      >
        Use Adapter
      </button>
      <div className="status">{status}</div>
    </fieldset>
  );
}
