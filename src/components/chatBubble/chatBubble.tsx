import './styles.css'

interface Props {
    text: string,
    chat_style: string
}

const ChatBubble: React.FC<Props> = ({text, chat_style}) => {
    return (
        <div className={chat_style}>
            {text}
        </div>
    )
}

export default ChatBubble;