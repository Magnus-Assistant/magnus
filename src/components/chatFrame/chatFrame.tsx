import { useEffect, useState } from "react";
import MagnusChatBubble from "../magnusChatBubble/magnusChatBubble";
import UserChatBubble from "../userChatBubble/chatBubble"
import "./styles.css"

// This chatFrame will contain the chat bubbles from the user and from magnuss

export interface Message {
    type: 'magnus' | 'user'; 
    text: string 
};

export interface ChatFrameProps {
    initialMessages: Message[];
}

export function scrollToBottom() {
    const frame = document.getElementById('ChatFrame');

    // if we have a frame make sure that it always renders at the newest message
    if(frame){
        console.log("we have a chat frame");
        frame.scrollTop = frame.scrollHeight;
    }
}

const ChatFrame: React.FC<ChatFrameProps> = ({ initialMessages }) => {
    const [messages, setMessages] = useState<{ type: 'magnus' | 'user'; text: string }[]>([]);

    useEffect(() => {
        // Initialize messages with initialMessages when provided
        if (initialMessages) {
          setMessages(initialMessages);
          setTimeout(scrollToBottom, 0);
        }
      }, [initialMessages]);

    return (
        <div className="ChatFrame" id="ChatFrame">
            {
            messages?.map((message, index) => (
                message.type === 'magnus' ? (
                    <MagnusChatBubble key={index} magnusText={message.text}></MagnusChatBubble>
                ) : (
                    <UserChatBubble key={index} userText={message.text}></UserChatBubble>
                )
            ))}
        </div>
    )
}

export default ChatFrame;