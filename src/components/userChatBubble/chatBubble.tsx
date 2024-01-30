import "./styles.css"

interface Props {
    userText: string
}

const UserChatBubble: React.FC<Props> = ({userText}) => {
    return (
        <div className="userChatBubble">
            {userText}
        </div>
    )
}

export default UserChatBubble;