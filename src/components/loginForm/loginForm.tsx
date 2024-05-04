import { useAuth0 } from "@auth0/auth0-react";
import "./styles.css"

interface Props {
    onPreviewClick: () => void
}

const LoginForm: React.FC<Props> = ({ onPreviewClick }) => {
    const { loginWithRedirect } = useAuth0();

    return (
        <div className="signinwrapper">
            <div className="signintextwrapper">
                <h3 className="signinheader">Welcome to Magnus!</h3>
                <div className="columncontainer">
                    <div className="column">
                        <u><h3>Featured</h3></u>
                        <ul>
                            <li>Free access to GPT-4</li>
                            <li>Text-to-Speech responses</li>
                            <li>Last 2 message exchanges included in context</li>
                            <li>Access to newer features coming soon...</li>
                        </ul>
                    </div>
                    <div className="column">
                        <u><h3>Preview</h3></u>
                        <ul>
                            <li>Free access to GPT-4</li>
                            <li>Text-to-Speech responses</li>
                            <li>No previous messages included in context</li>
                        </ul>
                    </div>
                </div>
            </div>
            <div className="signinbuttonwrapper">
                <button className="signupbutton" onClick={() => loginWithRedirect()}>
                    Sign In
                </button>
                <button className="previewbutton" onClick={onPreviewClick}>
                    Try Preview
                </button>
            </div>
        </div>
    )
}

export default LoginForm