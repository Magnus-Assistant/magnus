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

  async function startStream() {
    await invoke('start_stream');
  }

  async function stopStream() {
    await invoke("stop_stream");
  }

  return (
    <div className="container">
      <button onClick={startStream}>Start Audio Stream</button>
      <button onClick={stopStream}>Stop Audio Stream</button>
      <button onClick={createMessageThread}>Create Message Thread</button>
      <button onClick={printMessages}>Print Messages</button>
      <form onSubmit={createMessage}>
        <input type="text" value={text} onChange={changeText} />
        <button type="submit">Send</button>
      </form>
    </div>
  )
}

export default App;
