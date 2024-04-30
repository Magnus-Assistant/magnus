import { useAuth0 } from "@auth0/auth0-react";
import "./styles.css"

const LoginForm: React.FC = () => {

    const { loginWithRedirect, isAuthenticated, user } = useAuth0();

    console.log(user?.sub)
    console.log("Are we Authenticated: " + isAuthenticated)

    return (
        <div className="signinwrapper">
            <div className="signintextwrapper">
                <p>Some Text</p>
            </div>
            <div className="signinbuttonwrapper">
            <button className="signupformbutton" onClick={() => loginWithRedirect()}>
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