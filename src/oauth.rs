use std::env;
extern crate google_drive3 as drive;

use anyhow::{Ok, Result};
use drive::{
    hyper_rustls::HttpsConnector,
    oauth2::{
        authenticator::Authenticator, parse_application_secret, ApplicationSecret,
        InstalledFlowAuthenticator, InstalledFlowReturnMethod,
    },
};
use serde_json::from_str;

pub struct OAuthCredentialManager {
    pub credential: ApplicationSecret,
    pub connector: Option<Authenticator<HttpsConnector<drive::hyper::client::HttpConnector>>>,
}

impl OAuthCredentialManager {
    // Gets the OAuth credentails from the env which is stored in a compact string format
    pub fn try_new_from_env_string(env_path: String) -> Result<Self> {
        let oath_string = env::var(env_path).expect("Unable to get env");
        let credential = parse_application_secret(oath_string).unwrap();
        Ok(Self {
            credential,
            connector: None,
        })
    }

    // Gets the OAuth credentials from the env which is stored in broken format
    pub fn try_new_from_env() -> Result<Self> {
        let client_id = env::var(String::from("CLIENT_ID")).expect("Unable to client_id");
        let client_secret =
            env::var(String::from("CLIENT_SECRET")).expect("Unable to client_secret");
        let token_uri = env::var(String::from("TOKEN_URI")).expect("Unable to get token_uri");
        let auth_uri = env::var(String::from("AUTH_URI")).expect("Unable to get auth_uri");
        let redirect_uris: Vec<String> = from_str(
            env::var(String::from("REDIRECT_URIS"))
                .expect("Unable to get redirect_uris")
                .as_str(),
        )
        .expect("Cant marshall redirect uris");
        let mut project_id: Option<String> = Some(String::new());
        if env::var(String::from("PROJECT_ID")).is_ok() {
            project_id = Some(env::var(String::from("PROJECT_ID")).unwrap());
        }
        let mut client_email: Option<String> = Some(String::new());
        if env::var(String::from("CLIENT_EMAIL")).is_ok() {
            client_email = Some(env::var(String::from("CLIENT_EMAIL")).unwrap());
        }
        let mut auth_provider_x509_cert_url: Option<String> = Some(String::new());
        if env::var(String::from("AUTH_PROVIDER_URL")).is_ok() {
            auth_provider_x509_cert_url =
                Some(env::var(String::from("AUTH_PROVIDER_URL")).unwrap());
        }
        let mut client_x509_cert_url: Option<String> = Some(String::new());
        if env::var(String::from("CLIENT_CERT_URL")).is_ok() {
            client_x509_cert_url = Some(env::var(String::from("CLIENT_CERT_URL")).unwrap());
        }

        Ok(Self {
            credential: ApplicationSecret {
                client_id,
                client_secret,
                token_uri,
                auth_uri,
                redirect_uris,
                project_id,
                client_email,
                auth_provider_x509_cert_url,
                client_x509_cert_url,
            },
            connector: None,
        })
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        let auth: Authenticator<HttpsConnector<drive::hyper::client::HttpConnector>> =
            InstalledFlowAuthenticator::builder(
                self.credential.clone(),
                InstalledFlowReturnMethod::HTTPRedirect,
            )
            .persist_tokens_to_disk("tokencache.json")
            .build()
            .await
            .unwrap();

        self.connector = Some(auth);

        Ok(())
    }

    pub async fn default_initialize() -> Result<Self> {
        let mut cred_manager = Self::try_new_from_env_string(String::from("OAUTH_CREDENTIALS"));

        if !cred_manager.is_ok() {
            cred_manager = Self::try_new_from_env();
        }

        let mut cm = cred_manager.expect("Not able to get credentials");
        cm.authenticate().await.expect("Not able to authenticate");

        Ok(cm)
    }
}
