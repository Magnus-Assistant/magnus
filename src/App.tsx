import { invoke } from "@tauri-apps/api/tauri"
import { listen } from '@tauri-apps/api/event'
import React, { FormEvent, useEffect, useRef, useState } from 'react'
import ChatFrame, { Message, scrollToBottom } from "./components/chatFrame/chatFrame";
import SettingsModal from "./components/settingsModal/settingsModal";

type Payload = {
  message: string;
};

function App() {
  const [text, setText] = useState<string>('')
  const [messages, setMessages] = useState<Message[]>([]);
  const [shouldMic, setShouldMic] = useState(true);
  const [typing, setTyping] = useState(false)
  const [showSettings, setShowSettings] = useState(false)

  const changeText = (event: React.ChangeEvent<HTMLInputElement>) => {
    setText(event.target.value)
  }

  const handleFormSubmit = (event: FormEvent) => {
    event.preventDefault()
    //create a new message and set the message in local state
    if (text) {
      setTimeout(scrollToBottom, 30);
      setTyping(true)
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
            setTyping(false)
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
      <ChatFrame initialMessages={messages} typing={typing}></ChatFrame>
      <form onSubmit={handleFormSubmit} className="bottomBar">
        <button id="settingsButton" type="button" onClick={() => {setShowSettings(true)}}>
          <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 16 16">
            <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492M5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0"/>
            <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115z"/>
          </svg>
        </button>
        <button id="micButton" type="button" onClick={handlMicClick}>
          <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 16 16">
            <path d="M3.5 6.5A.5.5 0 0 1 4 7v1a4 4 0 0 0 8 0V7a.5.5 0 0 1 1 0v1a5 5 0 0 1-4.5 4.975V15h3a.5.5 0 0 1 0 1h-7a.5.5 0 0 1 0-1h3v-2.025A5 5 0 0 1 3 8V7a.5.5 0 0 1 .5-.5"/>
            <path d="M10 8a2 2 0 1 1-4 0V3a2 2 0 1 1 4 0zM8 0a3 3 0 0 0-3 3v5a3 3 0 0 0 6 0V3a3 3 0 0 0-3-3"/>
          </svg>
        </button>
        <input value={text} onChange={changeText} />
        <button type="submit" id="submitButton">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 16 16">
            <path d="M15.854.146a.5.5 0 0 1 .11.54l-5.819 14.547a.75.75 0 0 1-1.329.124l-3.178-4.995L.643 7.184a.75.75 0 0 1 .124-1.33L15.314.037a.5.5 0 0 1 .54.11ZM6.636 10.07l2.761 4.338L14.13 2.576zm6.787-8.201L1.591 6.602l4.339 2.76z"/>
          </svg>
        </button>
      </form>
      <SettingsModal show={showSettings} onClose={() => {setShowSettings(false)}} />
    </div>
  )
}

export default App;
