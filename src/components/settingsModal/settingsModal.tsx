import React, { useEffect, useState } from 'react'
import './styles.css'
import { invoke } from "@tauri-apps/api/tauri"
import CicularLoading from "../circularLoading/circularLoading"
import LogoutButton from '../logoutButton/logoutButton';
import UserIcon from '../userIcon/userIcon';

interface ModalProps {
  show: boolean;
  onClose: () => void;
}

interface Permissions {
  [key: string]: boolean;
}

interface AudioDeviceSelection {
  devices: string[];
  selected: string;
}

const SettingsModal: React.FC<ModalProps> = ({ show, onClose }) => {
  if (!show) return null;

  const [permissions, setPermissions] = useState<Permissions>({})
  const [audioInputDeviceSelection, setAudioInputDeviceSelection] = useState<AudioDeviceSelection>({ devices: [], selected: "" })
  const [audioOutputDeviceSelection, setAudioOutputDeviceSelection] = useState<AudioDeviceSelection>({ devices: [], selected: "" })
  const [inputDeviceSelected, setInputDeviceSelected] = useState<String>("")
  const [outputDeviceSelected, setOutputDeviceSelected] = useState<String>("")

  // get all info needed for the settings modal
  useEffect(() => {
    invoke("get_permissions").then((permissions: any) => {
      setPermissions(permissions as Permissions)
    })

    // refresh audio input and ouput devices every 2 seconds
    const refreshAudioDevices = setInterval(() => {
      invoke("get_audio_input_devices").then((audioInputDeviceSelection: any) => {
        setInputDeviceSelected(audioInputDeviceSelection.selected)
        setAudioInputDeviceSelection(audioInputDeviceSelection)
        // console.log(audioInputDeviceSelection.selected)
      })

      invoke("get_audio_output_devices").then((audioOutputDeviceSelection: any) => {
        setOutputDeviceSelected(audioOutputDeviceSelection.selected)
        setAudioOutputDeviceSelection(audioOutputDeviceSelection)
        // console.log(audioOutputDeviceSelection.selected)
      })
    }, 2000)

    // this isn't working correctly, interval remains even after modal is not shown
    return () => clearInterval(refreshAudioDevices)
  }, [])

  const handleToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    setPermissions({ ...permissions, [event.target.name]: event.target.checked });
  };

  useEffect(() => {
    async function updatePermissions() {
      if (Object.keys(permissions).length > 0) {
        await invoke("update_permissions", { permissions: permissions }).then(() => {
          console.log("updating permissions")
        })
      }
    }
    updatePermissions()
  }, [permissions])

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

  const handleAudioInputDeviceSelection = async (event: React.ChangeEvent<HTMLInputElement>) => {
    setInputDeviceSelected(event.target.value)
    await invoke("audio_input_device_selection", { deviceName: event.target.value })
  };

  const handleAudioOutputDeviceSelection = async (event: React.ChangeEvent<HTMLInputElement>) => {
    setOutputDeviceSelected(event.target.value)
    await invoke("audio_output_device_selection", { deviceName: event.target.value })
  }

  return (
    <div className="modal-backdrop">
      <div className="large-modal">
        <div className="modal-header">
          <h1>Settings</h1>
          <button onClick={onClose}>&times;</button>
        </div>
        <div className="modal-body">
          <div>
            <hr />
            Permissions
            <hr />
            {Object.entries(permissions).map(([name, value]) => (
              <div key={name} className="permissions">
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
            <hr />
            Audio
            <hr />
            {audioInputDeviceSelection.selected != "" ? (
              <div>
                <div className="section-title">Input</div>
                <div className='device-list'>
                  {audioInputDeviceSelection.devices.map((device, index) => (
                    <div key={device}>
                      <input
                        id={`inputDevice-${index}`}
                        type="radio"
                        value={device}
                        checked={inputDeviceSelected === device}
                        onChange={handleAudioInputDeviceSelection}
                      />
                      <label htmlFor={`inputDevice-${index}`}>
                        {device}
                      </label>
                    </div>
                  ))}
                </div>
                <div className="section-title">Output</div>
                <div className='device-list'>
                  {audioOutputDeviceSelection.devices.map((device, index) => (
                    <div key={device}>
                      <input
                        id={`outputDevice-${index}`}
                        type="radio"
                        value={device}
                        checked={outputDeviceSelected === device}
                        onChange={handleAudioOutputDeviceSelection}
                      />
                      <label htmlFor={`outputDevice-${index}`}>
                        {device}
                      </label>
                    </div>
                  ))}
                </div>
              </div>
            ) : (
              <CicularLoading />
            )}
          </div>
        </div>
        <div className='accountwrapper'>
          <UserIcon></UserIcon>
          <LogoutButton></LogoutButton>
        </div>
      </div>
    </div>
  );
};

export default SettingsModal;