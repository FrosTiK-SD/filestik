# FileSTiK

This is an open source project to handle Google Drive files as a backend server. This project aims to provide generic endpoints to handle Google Drive files fully writen in Rust [WIP]

## Why FilesTiK

When working with Google Drive APIs its very important to minimize the number of API calls due to rate-limiting as well as network overhead. FileSTiK tries to minimize all network overhead and can deliver responses faster than Google Drive API (in some cases) as it serves a cached response.

## Prerequisites

1. Rust Installed
2. Cargo Installed
3. GCP account
4. GhostScript Installed

## Getting started

1. Clone the repo
2. In the GCP console create a `web` `OAuth Client` in the credentials dashboard and store the json as a string in the environment.

This should look something like this.

```
export OAUTH_CREDENTIALS='{
  "web": {
    "client_id": "...",
    "project_id": "...",
    "auth_uri": "https://accounts.google.com/o/oauth2/auth",
    "token_uri": "https://oauth2.googleapis.com/token",
    "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
    "client_secret": "...",
    "redirect_uris": [
      "...",
      "http://127.0.0.1:61684"
    ],
    "javascript_origins": [
        ...
    ]
  }
}
'
```

:::note
Also note that `http://127.0.0.1:61684` should be whitelisted by the `OAuth client` in the `redirect_uris` in order to authorize the app
:::

3. Thats it. Now just run

```
cargo run
```

Your web server should get started at PORT: `8000`

## API Specs

[WIP]
