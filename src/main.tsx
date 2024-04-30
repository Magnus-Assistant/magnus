import React, { useEffect } from "react";
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
  let auth_domain = "";
  let auth_clientId = "";

  await invoke("get_auth_domain").then((domain: any) => {
    auth_domain = domain;
  })
  
  await invoke("get_auth_client_id").then((client: any) => {
    auth_clientId = client;
  })

  let secrets: Secrets = { domain: auth_domain, client: auth_clientId }
  return secrets
}

get_secrets().then((sec) => {
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
})
