import { invoke } from "@tauri-apps/api/tauri"
import "./App.css"
import React, { useState } from 'react'

function App() {
  const [text, setText] = useState<string>('')

  const changeText = (event: React.ChangeEvent<HTMLInputElement>) => {
    setText(event.target.value)
  }

  async function getClipboardText() {
    await invoke('get_clipboard_text')
  }

  async function captureScreen() {
    await invoke('capture_screen')
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

  return (
    <div className="container">
      <button onClick={getClipboardText}>Get Clipboard Text</button>
      <button onClick={captureScreen}>Capture Screen</button>
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
