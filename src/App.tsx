import { invoke } from "@tauri-apps/api/tauri"
import "./App.css"
import React, { useState } from 'react'

function App() {
  const [text, setText] = useState<string>('')

  const changeText = (event: React.ChangeEvent<HTMLInputElement>) => {
    setText(event.target.value)
  }

  async function createMessage() {
    await invoke('create_message', { message: text })
  }

  return (
    <div className="container">
      <form onSubmit={createMessage}>
        <input type="text" value={text} onChange={changeText} />
        <button type="submit">Send</button>
      </form>
      <div id="model_output"></div>
    </div>
  )
}

export default App;
