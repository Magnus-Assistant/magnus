import React, { useEffect, useState } from 'react'
import './styles.css'
// import { invoke } from "@tauri-apps/api/tauri"

interface ModalProps {
  show: boolean;
  onClose: () => void;
}

const SettingsModal: React.FC<ModalProps> = ({ show, onClose }) => {
  if (!show) return null;

  // TODO: get the users current permisions json
  const [toggles, setToggles] = useState({
    Location: false,
    Clipboard: false,
    Screenshot: false
  });

  const handleToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    setToggles({ ...toggles, [event.target.name]: event.target.checked });
  };

  useEffect(() => {
    // TODO: update the permissions.json
    console.log(toggles)
  }, [toggles])

  useEffect(() => {
    // when clicking anywhere except on the settings modal, close the modal
    const handleClickOutside = (event: MouseEvent) => {
      const modal = document.querySelector('.large-modal');
      if (modal && !modal.contains(event.target as Node)) {
        onClose();
      }
    };

    // add outside click listener if the settings modal is being showed
    if (show) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    // cleanup click listener on component unmount or for when show changes again
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [show]);

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
                  <label className="label" htmlFor={name}>{name}</label>
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
      </div>
    </div>
  );
};

export default SettingsModal;