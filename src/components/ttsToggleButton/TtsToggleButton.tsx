import "./styles.css"

interface ttsButtonProps {
    onClick: () => void;
}

const TtsButton: React.FC<ttsButtonProps> = ( { onClick } ) => {
    return (
        <button id="ttsButton" type="button" onClick={onClick}>
            TTS
        </button>
    )
}

export default TtsButton;