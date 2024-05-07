import { useAuth0 } from "@auth0/auth0-react";
import "./styles.css"

// I figured we could have this in the settings or something?
const LogoutButton: React.FC = () => {

    const { logout, user } = useAuth0();

    // render based on if we have a user or not
    return (
        <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
            {user ? (
                <button className="logoutbutton" onClick={() => logout({ logoutParams: { returnTo: window.location.origin } })}>
                    Sign Out
                </button>
            ) : (
                <button className="signinbutton" onClick={() => logout({ logoutParams: { returnTo: window.location.origin } })}>
                    Sign In
                </button>
            )}
        </div>
    )
}

export default LogoutButton