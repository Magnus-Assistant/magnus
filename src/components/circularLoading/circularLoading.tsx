import React from 'react';
import './styles.css'

interface Props {
  size: string
}

const CircularLoading: React.FC<Props> = (size) => {

  return (
    <div className="loader-container">
      <div className={size.size + "loader"}></div>
    </div>
  );
};

export default CircularLoading;