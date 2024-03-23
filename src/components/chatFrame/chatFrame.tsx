import { useEffect } from "react";
import ChatBubble from "../chatBubble/chatBubble"
import TypingIndicator from "../typingIndicator/typingIndicator";
import "./styles.css"

// This chatFrame will contain the chat bubbles from the user and from magnuss
export type Message = {
    type: 'magnus' | 'user';
    text: string
};

export interface ChatFrameProps {
    initialMessages: Message[];
}

export function scrollToBottom() {
    const frame = document.getElementById('ChatFrame');

    // if we have a frame make sure that it always renders at the newest message
    if (frame) {
        frame.scrollTop = frame.scrollHeight;
    }
}

const ChatFrame: React.FC<ChatFrameProps> = ({ initialMessages }) => {

    useEffect(() => {
        // Initialize messages with initialMessages when provided
        if (initialMessages) {
            setTimeout(scrollToBottom, 0);
        }

    }, [initialMessages]);

    return (
        <div className="ChatFrame" id="ChatFrame">
            {
                initialMessages?.map((message, index) => (
                    message.type === 'magnus' ? (
                        <ChatBubble key={index} text={message.text} chat_style="magnusChatBubble"></ChatBubble>
                    ) : (
                        <ChatBubble key={index} text={message.text} chat_style="userChatBubble"></ChatBubble>
                    )
                ))}
            <TypingIndicator />
        </div>
    )
}

export default ChatFrame;