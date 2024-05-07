import './styles.css';

const ContextLine: React.FC = () => {

  return (
    <>
      <div className='contextText'> Out of Context </div>
      <hr id='contextLine' />
      <div className='contextText'> In Context </div>
    </>
  )
}

export default ContextLine;