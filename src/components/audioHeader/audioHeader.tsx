import "./styles.css"
import RefreshIcon from "../../assets/RefreshIcon.png"

interface Props {
  onClickRefresh: () => void
}

// component for the audio settings since it contains a refresh button
const AudioHeader: React.FC<Props> = ({ onClickRefresh }) => {

  return (
    <div className='audioWrapper'>
      <div className="flex-container">
        <p className='audioTitle'>Audio</p>
      </div>
      <button className='refreshButton' onClick={onClickRefresh}>
        <img className='refreshImage' src={RefreshIcon} />
      </button>

    </div>
  )
}

export default AudioHeader