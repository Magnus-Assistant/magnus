import './styles.css'
import CodeBubble from "../codeBubble/codeBubble"
import Markdown from 'react-markdown';

interface Props {
    text: string,
    chat_style: string
}

const ChatBubble: React.FC<Props> = ({ text, chat_style }) => {
    let codeChunks = text.match(/```[^```]+```/g) || []
    let textChunks = text.split(/```[^```]+```/)
    textChunks = textChunks.filter(chunk => chunk.trim() !== '');

    return (
        <span>
            {textChunks.map((text, _) => {
                // Check if codeChunks has at least one element before shifting
                const codeChunk = codeChunks.length > 0 ? codeChunks.shift() : null;
    
                if (chat_style === "magnusChatBubble") {
                    return (
                        <>
                            <div className={chat_style}>
                                <Markdown className={"markdown"}>
                                    {text.trim()}
                                </Markdown>
                            </div>
                            {codeChunk && <CodeBubble codeChunk={codeChunk} />}
                        </>
                    );
                }
                else {
                    return (
                        <>
                            <div className={chat_style}>
                                {text.trim()}
                            </div>
                            {codeChunk && <CodeBubble codeChunk={codeChunk} />}
                        </>
                    );
                }
            })}
        </span>
    )
}

export default ChatBubble;