import './styles.css'
import { useState } from 'react'
import { Prism } from 'react-syntax-highlighter';
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism';

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
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 16 16" fill='lightgrey'>
                                    <path fill-rule="evenodd" d="M10.854 7.146a.5.5 0 0 1 0 .708l-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 9.793l2.646-2.647a.5.5 0 0 1 .708 0"/>
                                    <path d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1z"/>
                                    <path d="M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0z"/>
                                </svg>
                            ) : (
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 16 16" fill='lightgrey'>
                                    <path d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1z"/>
                                    <path d="M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0z"/>
                                </svg>
                            )
                        }
                    </button>
                </div>
                <div className='codeBlock'>
                    <Prism language={language} style={tomorrow}>
                        {code}
                    </Prism>
                </div>
            </div>
        </span>
    )
}

export default CodeBubble;