import { invoke } from "@tauri-apps/api/tauri"
import "./App.css"
import React, { useState } from 'react'

function App() {
  const [text, setText] = useState<string>('')

  const changeText = (event: React.ChangeEvent<HTMLInputElement>) => {
    setText(event.target.value)
  }

  async function createMessageThread() {
    await invoke('create_message_thread')
  }

  async function printMessages() {
    await invoke('print_messages')
  }

  async function createMessage() {
    await invoke('create_message', { message: text })
  }

  async function createMessageWithParam(text: string) {
    await invoke('create_message', { message: text })
  }

  async function startStream() {
    await invoke('start_stream');
  }

  async function stopStream() {
    await invoke("stop_stream");
    let text_output = await getStreamResults();
    console.log(text_output);
    if (text_output !== "") {
      await createMessageThread();
      await createMessageWithParam(text_output);
    }
  }

  async function getStreamResults(): Promise<string> {
    let text = "";
    let attempts = 0;
    while (text === null || text === "") {
      if (attempts > 5) {
        console.log("Failed to get stream data...")
        return ""
      }
      text = await invoke("get_stream_results")
      await new Promise((timeout) => setTimeout(timeout, 300));
      attempts++;
    }
    return text;
  }

  async function getSystemReport() {
    await invoke("get_system_report");
  }

  return (
    <div className="container">
      <button onClick={getSystemReport}>Get System Report</button>
      <button onClick={startStream}>Start Audio Stream</button>
      <button onClick={stopStream}>Stop Audio Stream</button>
      <button onClick={printMessages}>Print Messages</button>
      <form onSubmit={createMessage}>
        <input type="text" value={text} onChange={changeText} />
        <button type="submit">Send</button>
      </form>
      <div id="model_output"></div>
    </div>
  )
}

export default App;
