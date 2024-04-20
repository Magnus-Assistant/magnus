import './styles.css'
import CodeBubble from "../codeBubble/codeBubble"
import Markdown from 'react-markdown';

interface Props {
    text: string,
    chat_style: string
}

const ChatBubble: React.FC<Props> = ({ text, chat_style }) => {
    // split text into a list of strings of the text chunks and code snippets
    let chunks = text.split(/(?=\n`{3}.)|(?<=`{3})\n/).map(chunk => chunk.trim())

    return (
        <span>
            {chunks.map(chunk => {
                if (chat_style === "magnusChatBubble") {
                    if (chunk.startsWith("```") && chunk.endsWith("```")) {
                        return (
                            <CodeBubble codeChunk={chunk} />
                        )
                    }
                    else {
                        return (
                            <div className={chat_style}>
                                <Markdown className={"markdown"}>
                                    {chunk}
                                </Markdown>
                            </div>
                        )
                    }
                }
                else {
                    return (
                        <div className={chat_style}>
                            {chunk}
                        </div>
                    );
                }
            })}
        </span>
    )
}

export default ChatBubble;