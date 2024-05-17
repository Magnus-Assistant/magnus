import { invoke } from "@tauri-apps/api/tauri"
import { listen } from '@tauri-apps/api/event'
import React, { FormEvent, useEffect, useRef, useState } from 'react'
import ChatFrame, { Message } from "./components/chatFrame/chatFrame";
import SettingsModal from "./components/settingsModal/settingsModal";
import SettingsIcon from "./assets/SettingsIcon.svg"
import MicIcon from "./assets/MicIcon.svg"
import SendIcon from "./assets/SendIcon.svg"
import LoginForm from "./components/loginForm/loginForm";
import { useAuth0 } from "@auth0/auth0-react";
import CircularLoading from "./components/circularLoading/circularLoading";
import * as c from 'crypto-js';

type Payload = {
  message: string;
};

function App() {
  const [text, setText] = useState<string>('')
  const [messages, setMessages] = useState<Message[]>([]);
  const [shouldMic, setShouldMic] = useState(true);
  const [loading, setLoading] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [showPreview, setShowPreview] = useState(false);
  const formRef = useRef<HTMLFormElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const { isLoading, isAuthenticated, error, user, getIdTokenClaims } = useAuth0();

  let jwt = getIdTokenClaims().then(token => {
    console.log(token?.__raw)
    // invoke and save to a global variable (wrapped in an Option), for every request to the backend send the JWT as the Authenitcation header value example: Bearer *JWT*
  })

  function generateHash(input: string | undefined): string {
    
    // if we have an value, hash it
    if (input) {
      const hash = c.SHA256(input);
      const hashedString = hash.toString(c.enc.Hex);
      return hashedString;
    }
    // if it is undefined return an empty string and the backend will handle the input validation
    return ""
  }

  // set the auth status on the backend
  // can return false for isAuthenticated at first so its safer to run this each time it changes
  // also if something were to happen where they are no longer auth'd the backend would be informed as well
  useEffect(() => {
    if (isAuthenticated) {
      let created = new Date();
      invoke("set_is_signed_in", { isSignedIn: true })

      // create user on our backend if it doesnt exist
      invoke("create_user", {
        user: {
          user_id: generateHash(user?.email),
          username: user?.given_name ? user?.given_name : user?.nickname,
          email: user?.email,
          created_at: created.toLocaleString()
        },
      });
      console.log("We are using the authenticated mode")

    } else {

      invoke("set_is_signed_in", { isSignedIn: false })
      console.log("We are using the unauthenticated preview")
    }
  }, [isAuthenticated])

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
        //dont use the microphone   
        runConversationFlow(false);
        setText('');
      }
    }
  };

  const handleMicClick = async () => {
    const button = document.getElementById('micButton');
    if (button) {
      await invoke('get_permissions').then((permissions: any) => {
        if (permissions['Microphone']) {
          if (shouldMic) {
            setShouldMic(false)
            button.style.filter = "invert(100%)"
            //use the microphone
            runConversationFlow(true);
            console.log("Collecting Audio")
          }
          else {
            setShouldMic(true)
            button.style.filter = "invert(0%)"
            console.log("Audio Collecting Turned Off")
          }
        }
      })
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

  function canUseInput(canToggle: boolean) {
    let textBox = document.getElementById("magnus-textbox") as HTMLTextAreaElement
    textBox ? textBox.disabled = !canToggle : {}
    // click back into input field for convenience
    if (textBox && canToggle) {
      textareaRef.current?.focus()
    }

    let micButton = document.getElementById("micButton") as HTMLButtonElement
    micButton ? micButton.disabled = !canToggle : {}
    canToggle ? micButton.style.cursor = 'pointer' : micButton.style.cursor = 'default'

    let submitButton = document.getElementById("submitButton") as HTMLButtonElement
    submitButton ? submitButton.disabled = !canToggle : {}
    canToggle ? submitButton.style.cursor = 'pointer' : submitButton.style.cursor = 'default'
  }

  const hasBeenCalledRef = useRef(false);
  useEffect(() => {
    if (!hasBeenCalledRef.current) {
      hasBeenCalledRef.current = true

      const startListeners = async () => {
        await listen<Payload>("user", (response) => {
          if (typeof (response.payload.message) === 'string') {

            // disallow all input while magnus responds
            canUseInput(false);

            setLoading(true)
            const newMessage: Message = { type: 'user', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, newMessage])
          }

          // reset the state of the mic button if we get a transcription from the user back
          setShouldMic(true)
          const button = document.getElementById('micButton');
          if (button) {
            button.style.filter = "invert(0%)"
          }
        });

        await listen<Payload>("magnus", (response) => {
          if (typeof (response.payload.message) === 'string') {

            // allow user input after magnus has responded
            canUseInput(true);

            setLoading(false)
            const newMessage: Message = { type: 'magnus', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, newMessage])
          }
        })

        // listen for when magnus takes an action
        await listen<Payload>("action", (response) => {
          if (typeof (response.payload.message) === "string") {
            const actionMessage: Message = { type: 'magnus', text: response.payload.message }
            setMessages((prevMessages) => [...prevMessages, actionMessage])
          }
        })
      }
      startListeners();
    }
  }, [])

  const scrollToBottom = () => {
    window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' })
  }

  // scroll to bottom on new messages
  useEffect(() => {
    scrollToBottom()
  }, [messages])

  if (!isAuthenticated && !isLoading && !showPreview) {
    return (
      <LoginForm onPreviewClick={() => setShowPreview(true)}></LoginForm>
    )
  }

  if (isLoading) {
    return (
      <div className="container">
        <CircularLoading size="large" />
      </div>
    )
  }

  if (error) {
    return (
      <p>There was an error logging you in :(</p>
    )
  }

  if (!error && !isLoading) {
    return (
      <div className="container">
        <ChatFrame initialMessages={messages} loading={loading} isSignedIn={isAuthenticated}></ChatFrame>
        <form ref={formRef} onSubmit={handleFormSubmit} className="bottomBar">
          <button id="settingsButton" type="button" onClick={() => { setShowSettings(true) }}>
            <img src={SettingsIcon} />
          </button>
          <button id="micButton" type="button" onClick={handleMicClick}>
            <img src={MicIcon} />
          </button>
          <textarea ref={textareaRef} id="magnus-textbox" value={text} onChange={event => { setText(event.target.value) }} onKeyDown={handleKeyDown} />
          <button type="submit" id="submitButton">
            <img src={SendIcon} />
          </button>
        </form>
        <SettingsModal show={showSettings} onClose={() => { setShowSettings(false) }} />
      </div>
    )
  }
}

export default App;
