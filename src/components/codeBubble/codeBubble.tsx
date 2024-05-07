import './styles.css'
import { useState } from 'react'
import { Prism } from 'react-syntax-highlighter';
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism';
import ClipboardIcon from "../../assets/ClipboardIcon.svg"
import ClipboardConfirmedIcon from "../../assets/ClipboardConfirmedIcon.svg"

interface Props {
    codeChunk: string
}

const CodeBubble: React.FC<Props> = ({ codeChunk }) => {
    const [copied, setCopied] = useState(false);

    // remove the first and last lines
    const code = codeChunk.trim().split('\n').slice(1, -1).join('\n')

    // extract the coding language
    const lang = codeChunk.match(/```.+\n/);
    const language = lang ? lang[0].substring(3).trim() : '';

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(code);
            setCopied(true)
            setTimeout(() => {
                setCopied(false)
            }, 4000)

        }
        catch (err) {
            console.log('no copy! :(' + err)
        }
    }

    const customStyle = {
        backgroundColor: window.matchMedia('(prefers-color-scheme: dark)').matches ? '#3f3f3f' : 'white'
    };
    
    return (
        <span>
            <div className='codeBubble'>
                <div className='codeBubbleHeader'>
                    <div className='codeLanguage'>
                        {language}
                    </div>
                    <button className='copyButton' onClick={handleCopy}>
                        {
                            copied ? (
                                <img src={ClipboardConfirmedIcon} />
                            ) : (
                                <img src={ClipboardIcon} />
                            )
                        }
                    </button>
                </div>
                <div className='codeBlock'>
                    <Prism language={language} style={tomorrow} customStyle={customStyle}>
                        {code}
                    </Prism>
                </div>
            </div>
        </span>
    )
}

export default CodeBubble;