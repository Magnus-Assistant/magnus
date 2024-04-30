import { useAuth0 } from "@auth0/auth0-react";
import "./styles.css"

const LoginForm: React.FC = () => {

    const { loginWithRedirect } = useAuth0();

    return (
        <div className="signinwrapper">
            <div className="signintextwrapper">
                <h3 className="signinheader">Welcome to Magnus!</h3>
                <p className="signintext">If you want to use the full product please select "<b>Sign In</b>" and create an account using auth0.</p>
                <p className="signintext">If you don't want to create an account right now you can use the demo version by selecting "<b>Use Demo</b>"</p>
            </div>
            <div className="signinbuttonwrapper">
                <button className="signupformbutton" style={{ backgroundColor: "#157DEC" }} onClick={() => loginWithRedirect()}>
                    Sign In
                </button>
                <button className="signupformbutton">
                    Use Demo
                </button>
            </div>
        </div>
    )
}

export default LoginForm