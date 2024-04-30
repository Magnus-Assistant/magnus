import { useAuth0 } from "@auth0/auth0-react";


export interface Props {
    shouldShow: boolean;
  }
  

const LoginForm: React.FC<Props> = (shouldShow) => {

    const { loginWithRedirect, logout, isLoading, isAuthenticated } = useAuth0();

    console.log("Are we Authenticated: " + isAuthenticated)

    return !isAuthenticated ? (
        <button onClick={() => loginWithRedirect()}>
            Sign In
        </button>
    ) : (
        <button onClick={() => logout({ logoutParams: { returnTo: window.location.origin } })}>
            Sign Out
        </button>
    )
}

export default LoginForm