import { invoke } from "@tauri-apps/api/tauri"
import "./App.css"
import React, { FormEvent, useState } from 'react'
import ChatFrame, { Message, scrollToBottom } from "./components/chatFrame/chatFrame";
import TtsButton from "./components/ttsToggleButton/TtsToggleButton";

function App() {
  const [text, setText] = useState<string>('')
  const [messages, setMessages] = useState<{ type: 'magnus' | 'user'; text: string }[]>([]);
  const [shouldTts, setShouldTts] = useState(false);

  const changeText = (event: React.ChangeEvent<HTMLInputElement>) => {
    setText(event.target.value)
  }

  const handleFormSubmit = (event: FormEvent) => {
    event.preventDefault()
    //create a new message and set the message in local state
    if (text) {
      const newMessage: Message = { type: 'user', text: text }
      setMessages((prevMessages) => [...prevMessages, newMessage])
      // add brief timeout so that the text can render. Then scroll down
      setTimeout(scrollToBottom, 30);

      //clear input field      
      createMessage();
      setText('');
    }
  };

  const handleTtsClick = () => {
    const button = document.getElementById('ttsButton');
    if (button) {
    if (shouldTts) {
      setShouldTts(false)
      button.style.color = "white";
      button.style.backgroundColor = '#0f0f0f'
    } else {
      setShouldTts(true)
      button.style.color = "black";
      button.style.backgroundColor = 'red';
    }
  }
}

  async function createMessage() {
    await invoke('create_message', { message: text, hasTts: shouldTts })
      .then((response) => {
        if (typeof (response) === 'string') {
          const newMessage: Message = { type: 'magnus', text: response }
          console.log(newMessage)
          setMessages((prevMessages) => [...prevMessages, newMessage])
          console.log(messages);
        }
      })
  }

  return (
    <div className="container">
      <ChatFrame initialMessages={messages}></ChatFrame>
      <form onSubmit={handleFormSubmit} style={{ justifyContent: 'center', marginTop: 10, marginBottom: 10 }}>
        <TtsButton onClick={handleTtsClick}></TtsButton>
        <input className="userTextBox" id="userTextBox" type="text" value={text} onChange={changeText} />
        <button type="submit">Send</button>
      </form>
      <div id="model_output"></div>
    </div>
  )
}

export default App;
