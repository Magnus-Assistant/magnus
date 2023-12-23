import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {

  async function consolePrint() {
    await invoke('my_custom_command');
  }

  async function startListener() {
    let result = await invoke('start_listener');
    console.log(typeof result)
  }

  return (
    <div className="container">
      <button onClick={consolePrint}>Print To Console</button>
      <button onClick={startListener}>Start Listener</button>
    </div>
  );
}

export default App;
