import { BaseDirectory, readTextFile } from "@tauri-apps/api/fs";

export function GraphControls() {
  function refreshGraph(e: React.MouseEvent<HTMLButtonElement, MouseEvent>) {
    e.preventDefault();
    (async () => {
      const data = await readTextFile("bt-data.json", {
        dir: BaseDirectory.AppData,
      });
      console.log(data);
    })();
  }
  return (
    <fieldset>
      <legend>Graph</legend>
      <select>
        <option>Graph 1</option>
        <option>Graph 2</option>
      </select>
      <button onClick={refreshGraph}>🔃</button>
    </fieldset>
  );
}
