import ChatBubble from "../chatBubble/chatBubble"
import LoadingIndicator from "../loadingIndicator/loadingIndicator";
import "./styles.css"

// This chatFrame will contain the chat bubbles from the user and from magnuss
export type Message = {
    type: 'magnus' | 'user';
    text: string
};

export interface ChatFrameProps {
    initialMessages: Message[];
    loading: boolean;
}


const ChatFrame: React.FC<ChatFrameProps> = ({ initialMessages, loading }) => {
    return (
        <div className="ChatFrame" id="ChatFrame">
            {
                initialMessages?.map((message, index) => (
                    message.type === 'magnus' ? (
                        <ChatBubble key={index} text={message.text} chat_style="magnusChatBubble"></ChatBubble>
                    ) : (
                        <ChatBubble key={index} text={message.text} chat_style="userChatBubble"></ChatBubble>
                    )
                ))
            }
            <LoadingIndicator loading={loading} />
        </div>
    )
}

export default ChatFrame;