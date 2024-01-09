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
  }

  async function diplayStreamResults() {
    const result = await invoke("get_stream_results")

    if (typeof result === 'string') {
      const modelTextElement = document.getElementById('model_output')
      if (modelTextElement) {
        modelTextElement.innerText = result;
      }
    } else {
      const modelTextElement = document.getElementById('model_output')
      if (modelTextElement) {
        modelTextElement.innerText = "Loading Results...";
      }
    }
  }

  async function getStreamResults(): Promise<string> {
    const result = await invoke("get_stream_results")

    if (typeof result === 'string') {
      return result;
    } else {
      return "";
    }
  }


  async function useGPTwithAudio() {
    await createMessageThread();
    const text = await getStreamResults();
    await createMessageWithParam(text);
  }

  return (
    <div className="container">
      <button onClick={startStream}>Start Audio Stream</button>
      <button onClick={stopStream}>Stop Audio Stream</button>
      <button onClick={diplayStreamResults}>Get Stream Results</button>
      <button onClick={useGPTwithAudio}>Use ChatGPT With Audio</button>
      <button onClick={createMessageThread}>Create Message Thread</button>
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
