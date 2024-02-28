import './styles.css'

interface Props {
    text: string,
    chat_style: string
}

const ChatBubble: React.FC<Props> = ({ text, chat_style }) => {
    return (
        <span>
            <div className={chat_style}>
                {text}
            </div>
        </span>
    )
}

export default ChatBubble;