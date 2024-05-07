import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { Auth0Provider } from "@auth0/auth0-react";
import { invoke } from "@tauri-apps/api";

interface Secrets {
  domain: string;
  client: string;
}


async function get_secrets(): Promise<Secrets> {

  let secrets: Secrets = { domain: "", client: "" }

  await invoke("get_auth_domain").then((domain: any) => {
    secrets.domain = domain;
  })

  await invoke("get_auth_client_id").then((client: any) => {
    secrets.client = client;
  })

  return secrets
}

//this kinda protects us by not letting us do anything if it can't pull the secrets
get_secrets().then((sec) => {
  if (sec.client.length > 0 && sec.domain.length > 0) {
    ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
      <React.StrictMode>
        <Auth0Provider // configure auth0 in the applications start point
          domain={sec.domain}
          clientId={sec.client}
          authorizationParams={{
            redirect_uri: window.location.origin
          }}>
          <App />
        </Auth0Provider>
      </React.StrictMode>,
    );
  }
})
