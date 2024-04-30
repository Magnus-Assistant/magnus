import { useAuth0 } from "@auth0/auth0-react";


export interface Props {
    shouldShow: boolean;
  }
  

const LoginForm: React.FC<Props> = (shouldShow) => {

    const { logout } = useAuth0();

    return (
        <button onClick={() => logout({ logoutParams: { returnTo: window.location.origin } })}>
            Sign Out
        </button>
    )
}

export default LoginForm