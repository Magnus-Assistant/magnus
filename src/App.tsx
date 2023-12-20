import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {

  async function consolePrint() {
    await invoke('my_custom_command');
  }

  async function startModel() {
    await invoke('start_model');
  }

  return (
    <div className="container">
      <button onClick={consolePrint}>Print To Console</button>
      <button onClick={startModel}>Start Vosk Model</button>
    </div>
  );
}

export default App;
