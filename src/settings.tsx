import { useState } from "react";

export function Settings() {
  const [show, setShow] = useState(false);

  return show ? (
    <div id="settings">
      <h2>Settings</h2>
      <button onClick={() => setShow(false)}>X</button>
      <button>Set Data Path</button>
    </div>
  ) : (
    <GearIcon onClick={() => setShow(true)} />
  );
}

function GearIcon({ onClick }: { onClick: () => void }) {
  return <button onClick={onClick}>⚙️</button>;
}
