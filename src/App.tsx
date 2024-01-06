import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {

  async function consolePrint() {
    await invoke('my_custom_command');
  }

  async function startModel() {
    await invoke('start_model');
  }

  async function startStream() {
    await invoke('start_stream');
  }

  async function stopStream() {
    await invoke("stop_stream");
  }

  return (
    <div className="container">
      <button onClick={consolePrint}>Print To Console</button>
      <button onClick={startModel}>Start Vosk Model</button>
      <button onClick={startStream}>Start Audio Stream</button>
      <button onClick={stopStream}>Stop Audio Stream</button>
    </div>
  );
}

export default App;
