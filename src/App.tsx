import { invoke } from "@tauri-apps/api/tauri"
import { listen } from '@tauri-apps/api/event'
import React, { FormEvent, useEffect, useRef, useState } from 'react'
import ChatFrame, { Message, scrollToBottom } from "./components/chatFrame/chatFrame";

type Payload = {
  message: string;
};

function App() {
  const [text, setText] = useState<string>('')
  const [messages, setMessages] = useState<Message[]>([]);
  const [shouldMic, setShouldMic] = useState(true);

  const changeText = (event: React.ChangeEvent<HTMLInputElement>) => {
    setText(event.target.value)
  }

  const handleFormSubmit = (event: FormEvent) => {
    event.preventDefault()
    //create a new message and set the message in local state
    if (text) {
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
      await invoke('run_conversation_flow', { userMessage: text })
      // just call the backend function, the chatbubble text is handled over a listener
    } else {
      await invoke('run_conversation_flow', { userMessage: null })
    }
  }

  const hasBeenCalledRef = useRef(false);
  useEffect(() => {
    if (!hasBeenCalledRef.current) {
      hasBeenCalledRef.current = true

      const grabTranscription = async () => {
        await listen<Payload>("user", (response) => {
          if (typeof (response.payload.message) === 'string') {
            const newMessage: Message = { type: 'user', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, newMessage])
          }

          //reset the state of the mic button if we get a transcription from the user back
          setShouldMic(true)
          const button = document.getElementById('micButton');
          if (button) {
            button.style.filter = "invert(0%)"
          }
        });

        await listen<Payload>("magnus", (response) => {
          if (typeof (response.payload.message) === 'string') {
            const newMessage: Message = { type: 'magnus', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, newMessage])
          }
        })
      }
      grabTranscription();
    }
  }, [])

  return (
    <div className="container">
      <ChatFrame initialMessages={messages}></ChatFrame>
      <form onSubmit={handleFormSubmit} style={{ justifyContent: 'center', marginTop: 10, marginBottom: 10 }}>
        <button id="micButton" type="button" onClick={handlMicClick}>-</button>
        <input className="userTextBox" id="userTextBox" type="text" value={text} onChange={changeText} />
        <button type="submit">Send</button>
      </form>
    </div>
  )
}

export default App;
