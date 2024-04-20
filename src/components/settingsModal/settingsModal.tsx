import React, { useEffect, useState } from 'react'
import './styles.css'
import { invoke } from "@tauri-apps/api/tauri"

interface ModalProps {
  show: boolean;
  onClose: () => void;
}

interface Permissions {
  [key: string]: boolean;
}

const SettingsModal: React.FC<ModalProps> = ({ show, onClose }) => {
  if (!show) return null;

  const [toggles, setToggles] = useState<Permissions>({})

  // get all info needed for the settings modal
  useEffect(() => {
    invoke("get_permissions").then((permissions: any) => {
      setToggles(permissions as Permissions)
    })

    invoke("get_audio_input_devices").then((audio_input_devices: any) => {
      console.log(audio_input_devices)
    })

    invoke("get_audio_output_devices").then((audio_output_devices: any) => {
      console.log(audio_output_devices)
    })
  }, [])

  const handleToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    setToggles({ ...toggles, [event.target.name]: event.target.checked });
  };

  useEffect(() => {
    async function updatePermissions() {
      await invoke("update_permissions", { permissions: toggles })
    }
    updatePermissions()
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