import ChatBubble from "../chatBubble/chatBubble"
import LoadingIndicator from "../loadingIndicator/loadingIndicator";
import ContextLine from "../contextLine/contextLine";
import "./styles.css"

// This chatFrame will contain the chat bubbles from the user and from magnuss
export type Message = {
    type: 'magnus' | 'user';
    text: string;
};

export interface ChatFrameProps {
    initialMessages: Message[];
    loading: boolean;
    isSignedIn: boolean;
}
const ChatFrame: React.FC<ChatFrameProps> = ({ initialMessages, loading, isSignedIn }) => {
    // get the number of exchanges in the conversation
    let count = 0
    let previousSender = 'user'
    for (let i = 0; i < initialMessages.length; i++) {
        if (initialMessages[i].type != previousSender) {
            count += 1
            previousSender = initialMessages[i].type
        }
    }
    let numExchanges = (count + 1) / 2

    // define the context line considering then number of exchanges, and if magnus has responded or not
    let contextLine = 0
    let numUserMessages = 0
    for (let i = initialMessages.length - 1; i >= 0; i--) {
        if (initialMessages[i].type == 'user') {
            numUserMessages += 1
        }
        if (numExchanges === Math.floor(numExchanges) && numUserMessages === 2) {
            contextLine = i
            break
        }
        else if (numExchanges !== Math.floor(numExchanges) && numUserMessages === 3) {
            contextLine = i
            break
        }
    }

    return (
        <div className="ChatFrame" id="ChatFrame">
            {
                initialMessages?.map((message, index) => (
                    <>
                        {index === contextLine && isSignedIn && <ContextLine />}
                        <ChatBubble key={index} text={message.text} chat_style={message.type === 'magnus' ? "magnusChatBubble" : "userChatBubble"} />
                    </>
                ))
            }
            <LoadingIndicator loading={loading} />
        </div>
    )
}

export default ChatFrame;