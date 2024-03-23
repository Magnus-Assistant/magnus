import React from 'react';
import './styles.css';

export interface Props {
  typing: boolean;
}

const TypingIndicator: React.FC<Props> = (typing) => {

  return typing.typing ? (
    <div className="typing-indicator">
      <span></span>
      <span></span>
      <span></span>
    </div>
  ) : null;
}

export default TypingIndicator;