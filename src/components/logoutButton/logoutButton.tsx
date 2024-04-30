import { useAuth0 } from "@auth0/auth0-react";

// I figured we could have this in the settings or something?
const LogoutButton: React.FC = () => {

    const { logout } = useAuth0();

    return (
        <button className="logoutbutton" onClick={() => logout({ logoutParams: { returnTo: window.location.origin } })}>
            Sign Out
        </button>
    )
}

export default LogoutButton