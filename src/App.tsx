import { invoke } from "@tauri-apps/api/tauri"
import React, { FormEvent, useState } from 'react'
import ChatFrame, { Message, scrollToBottom } from "./components/chatFrame/chatFrame";

function App() {
  const [text, setText] = useState<string>('')
  const [messages, setMessages] = useState<{ type: 'magnus' | 'user'; text: string }[]>([]);
  const [shouldMic, setShouldMic] = useState(false);

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
      console.log("Form Submit Messages: ", messages)
      setTimeout(scrollToBottom, 30);

      //dont use the microphone   
      runConversationFlow(false);
      setText('');
    }
  };

  const handlMicClick = () => {
    const button = document.getElementById('micButton');
    if (button) {
      if (shouldMic) {
        setShouldMic(false)
        button.style.filter = "invert(100%)"
        //use the microphone
        runConversationFlow(true);
        console.log("Collecting Audio")
      } else {
        setShouldMic(true)
        button.style.filter = "invert(0%)"
        console.log("Audio Collecting Turned Off")
      }
    }
  }

  async function runConversationFlow(use_mic?: boolean) {
    // if we shouldnt use the mic use the local text state contents
    if (!use_mic) {
    await invoke('run_conversation_flow', { userMessage: text})
      .then((response) => {
        if (typeof (response) === 'string') {
          const newMessage: Message = { type: 'magnus', text: response }
          setMessages((prevMessages) => [...prevMessages, newMessage])
          console.log("Create Message Command Messages: ", messages)
        }
      })
      // if we should use the mic don't pass in any text to use
    } else {
      await invoke('run_conversation_flow', { userMessage: null})
      .then((response) => {
        if (typeof (response) === 'string') {
          const newMessage: Message = { type: 'magnus', text: response }
          setMessages((prevMessages) => [...prevMessages, newMessage])
          console.log("Create Message Command Messages: ", messages)
        }
      })
    }
  }

  return (
    <div className="container">
      <ChatFrame initialMessages={messages}></ChatFrame>
      <form onSubmit={handleFormSubmit} style={{ justifyContent: 'center', marginTop: 10, marginBottom: 10 }}>
        <button id="micButton" type="button" onClick={handlMicClick}>-</button>
        <input className="userTextBox" id="userTextBox" type="text" value={text} onChange={changeText} />
        <button type="submit">Send</button>
      </form>
      <div id="model_output"></div>
    </div>
  )
}

export default App;
