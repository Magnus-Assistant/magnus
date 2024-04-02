import React, { useState } from 'react'
import './styles.css'
import { invoke } from "@tauri-apps/api/tauri"

interface ModalProps {
  show: boolean;
  onClose: () => void;
}

const SettingsModal: React.FC<ModalProps> = ({ show, onClose }) => {
  if (!show) return null;

  const [toggles, setToggles] = useState({
    toggle1: false,
    toggle2: false,
    toggle3: false,
  });

  const handleSubmit = async () => {
    // Send toggle values to Rust backend
    // await invoke('submit_toggle_data', { });
  };

  const handleToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    setToggles({ ...toggles, [event.target.name]: event.target.checked });
  };

  return (
    <div className="modal-backdrop">
      <div className="large-modal">
        <div className="modal-header">
          <h2>Settings</h2>
          <button onClick={onClose}>&times;</button>
        </div>
        <div className="modal-body">
            <form>
              {Object.entries(toggles).map(([name, value]) => (
                <div key={name} className="form-item">
                  <label htmlFor={name}>{name}</label>
                  <label className="switch">
                    <input
                      name={name}
                      type="checkbox"
                      checked={value}
                      onChange={handleToggle}
                    />
                    <span className="slider round"></span>
                  </label>
                </div>
              ))}
          </form>
        </div>
        <div className="modal-footer">
          <button onClick={onClose}>Close</button>
          <button onClick={handleSubmit}>Submit</button>
        </div>
      </div>
    </div>
  );
};

export default SettingsModal;