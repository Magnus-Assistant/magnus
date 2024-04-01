/*import React from 'react';
import Modal from 'react-bootstrap/Modal';
import 'bootstrap/dist/css/bootstrap.min.css'

interface Props {
  show: boolean;
  setShow: (arg0: boolean) => void;
}

const SettingsModal: React.FC<Props> = ({ show, setShow }) => {
  if (!show) {
    return null;
  }

  return (
    <Modal
        size="lg"
        show={show}
        onHide={() => setShow(false)}
        aria-labelledby="example-modal-sizes-title-lg"
    >
        <Modal.Header closeButton>
            <Modal.Title id="example-modal-sizes-title-lg">
                Large Modal
            </Modal.Title>
        </Modal.Header>
        <Modal.Body>...</Modal.Body>
    </Modal>
  );
};

export default SettingsModal;
*/
import React from 'react';
import './styles.css'; // Importing our custom CSS

interface ModalProps {
  show: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

const SettingsModal: React.FC<ModalProps> = ({ show, onClose, title, children }) => {
  if (!show) return null;

  return (
    <div className="modal-backdrop">
      <div className="large-modal">
        <div className="modal-header">
          <h5>{title}</h5>
          <button onClick={onClose}>&times;</button>
        </div>
        <div className="modal-body">{children}</div>
        <div className="modal-footer">
          <button onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  );
};

export default SettingsModal;