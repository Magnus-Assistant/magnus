import { useState } from 'react'
import ChatBubble from "../chatBubble/chatBubble"
import LoadingIndicator from "../loadingIndicator/loadingIndicator";
import ContextLine from "../contextLine/contextLine";
import "./styles.css"

// This chatFrame will contain the chat bubbles from the user and from magnuss
export type Message = {
    type: 'magnus' | 'user';
    text: string;
    excludeFromCount?: boolean;
};

export interface ChatFrameProps {
    initialMessages: Message[];
    loading: boolean;
    isSignedIn: boolean;
}
const ChatFrame: React.FC<ChatFrameProps> = ({ initialMessages, loading, isSignedIn }) => {
    let contextLineIndex = 0
    let nMessages = initialMessages.length

    if (isSignedIn && nMessages > 4) {
        contextLineIndex = nMessages - 4
    }
    if (nMessages % 2 === 1) {
        contextLineIndex -= 1
    }

    let messagesInContext = initialMessages.slice(contextLineIndex)

    for (let i = 0; i < messagesInContext.length; i++) {
        if (messagesInContext[i].excludeFromCount) {
            contextLineIndex -= 2
        }
    }

    console.log(contextLineIndex)

    return (
        <div className="ChatFrame" id="ChatFrame">
            {
                initialMessages?.map((message, index) => (
                    <>
                        {index === contextLineIndex && isSignedIn && <ContextLine />}
                        <ChatBubble key={index} text={message.text} chat_style={message.type === 'magnus' ? "magnusChatBubble" : "userChatBubble"} />
                    </>
                ))
            }
            <LoadingIndicator loading={loading} />
        </div>
    )
}

export default ChatFrame;