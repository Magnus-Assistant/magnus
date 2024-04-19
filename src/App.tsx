import { invoke } from "@tauri-apps/api/tauri"
import { listen } from '@tauri-apps/api/event'
import React, { FormEvent, useEffect, useRef, useState } from 'react'
import ChatFrame, { Message } from "./components/chatFrame/chatFrame";
import SettingsModal from "./components/settingsModal/settingsModal";
import SettingsIcon from "./assets/SettingsIcon.svg"
import MicIcon from "./assets/MicIcon.svg"
import SendIcon from "./assets/SendIcon.svg"

type Payload = {
  message: string;
};

function App() {
  const [text, setText] = useState<string>('')
  const [messages, setMessages] = useState<Message[]>([]);
  const [shouldMic, setShouldMic] = useState(true);
  const [loading, setLoading] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const formRef = useRef<HTMLFormElement>(null);

  const handleFormSubmit = (event: FormEvent) => {
    event.preventDefault()
    //create a new message and set the message in local state
    if (text) {
      setTimeout(scrollToBottom, 30);
      setLoading(true)
      //dont use the microphone   
      runConversationFlow(false);
      setText('');
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      if (text) {
        setTimeout(scrollToBottom, 30);
        setLoading(true)
        //dont use the microphone   
        runConversationFlow(false);
        setText('');
      }
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

      const startListeners = async () => {
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
            setLoading(false)
            const newMessage: Message = { type: 'magnus', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, newMessage])
          }
        })

        // listen for when magnus takes an action
        await listen<Payload>("action", (response) => {
          if (typeof (response.payload.message) === "string") {
            const actionMessage: Message = {type: 'magnus', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, actionMessage])  
          }
        })
      }
      startListeners();
    }
  }, [])

  const scrollToBottom = () => {
    window.scrollTo({top: document.body.scrollHeight, behavior: 'smooth'})
  }

  // scroll to bottom on new messages
  useEffect(() => {
    scrollToBottom()
  }, [messages])

  return (
    <div className="container">
      <ChatFrame initialMessages={messages} loading={loading}></ChatFrame>
      <form ref={formRef} onSubmit={handleFormSubmit} className="bottomBar">
        <button id="settingsButton" type="button" onClick={() => {setShowSettings(true)}}>
          <img src={SettingsIcon} />
        </button>
        <button id="micButton" type="button" onClick={handlMicClick}>
          <img src={MicIcon} />
        </button>
        <textarea value={text} onChange={event => {setText(event.target.value)}} onKeyDown={handleKeyDown}/>
        <button type="submit" id="submitButton">
          <img src={SendIcon} />
        </button>
      </form>
      <SettingsModal show={showSettings} onClose={() => {setShowSettings(false)}} />
    </div>
  )
}

export default App;
