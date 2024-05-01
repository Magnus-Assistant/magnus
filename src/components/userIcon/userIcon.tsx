import { useAuth0 } from "@auth0/auth0-react"
import "./styles.css"

const UserIcon: React.FC = () => {

    const { user } = useAuth0();

    return (
        <div className="userinfowrapper">
            <img src={user?.picture}></img>
            <div className="usernamewrapper">
                <p className="useremailtext">{user?.given_name ? user?.given_name : user?.nickname}</p>
            </div>
        </div>
    )
}

export default UserIcon