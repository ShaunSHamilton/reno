import { invoke } from "@tauri-apps/api/tauri";
import { emit, listen } from "@tauri-apps/api/event";
import { useState, useRef, useEffect } from "react";

export function PeripheralControls() {
  const [isSearching, setIsSearching] = useState(false);
  const [status, setStatus] = useState("Not Connected");
  const [peripheralId, setPeripheralId] = useState("");

  const selectEl = useRef<HTMLSelectElement | null>(null);

  useEffect(() => {
    let unlisten: () => void;
    (async () => {
      unlisten = await listen("DeviceDiscovered", (event) => {
        console.log(event);
        if (selectEl.current) {
          selectEl.current.innerHTML =
            "<option value=''>Select a Peripheral</option>";
          if (Array.isArray(event.payload)) {
            for (const peripheral of event.payload) {
              const option = document.createElement("option");
              option.text = peripheral as string;
              option.value = peripheral as string;
              selectEl.current.appendChild(option);
            }
          }
        }
      });
      if (isSearching) {
        setStatus("Searching for Peripherals");
        invoke("search_for_peripherals")
          .then(async () => {})
          .catch((e) => {
            console.error(e);
            setStatus("Error Searching for Peripherals");
          });
      } else {
        emit("stop-searching");
        setStatus(`Found ${selectEl.current?.options.length} Peripherals`);
      }

      return () => {
        unlisten();
      };
    })();
  }, [isSearching]);

  return (
    <fieldset>
      <legend>Peripheral</legend>
      <select
        ref={selectEl}
        onChange={(e) => {
          console.log(e);
          setPeripheralId(e.target.value);
        }}
      >
        <option value="">Select a Peripheral</option>
      </select>
      <button
        onClick={() => {
          setIsSearching(!isSearching);
        }}
      >
        {isSearching ? "Stop Searching" : "Search"} for Peripherals
      </button>
      <button
        onClick={() => {
          setStatus("Connecting to Peripheral");
          invoke("connect_to_peripheral", { id: peripheralId })
            .then(() => {
              setStatus("Connected to Peripheral");
            })
            .catch((e) => {
              console.error(e);
              setStatus("Error Connecting to Peripheral");
            });
        }}
      >
        Connect to Peripheral
      </button>
      <button
        onClick={() => {
          setStatus("Getting Latest Data");
          invoke("request_single_event")
            .then((d) => {
              console.log(d);
              setStatus(JSON.stringify(d, null, 2));
            })
            .catch((e) => {
              console.error(e);
              setStatus("Error Getting Latest Data");
            });
        }}
      >
        Get Latest
      </button>
      <button onClick={() => {}}>Record Data Stream</button>
      {/* Show errors or loading */}
      <div className="status">{status}</div>
    </fieldset>
  );
}
