import React from 'react';
import './styles.css';

export interface Props {
  loading: boolean;
}

const LoadingIndicator: React.FC<Props> = (loading) => {

  return loading.loading ? (
    <div className="loading-indicator">
      <span></span>
      <span></span>
      <span></span>
    </div>
  ) : null;
}

export default LoadingIndicator;