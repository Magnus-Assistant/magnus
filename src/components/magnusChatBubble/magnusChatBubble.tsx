import "./styles.css"

interface Props {
    magnusText: string
}

const MagnusChatBubble: React.FC<Props> = ({magnusText}) => {
    return (
        <div className="magnusChatBubble">
            {magnusText}
        </div>
    )
}

export default MagnusChatBubble;