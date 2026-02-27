use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl, reqwest::http_client,
};
use open;
use oauth2::TokenResponse;
use rusqlite::Connection;
use tiny_http::{Server, Response, Header};

use url::form_urlencoded;
use std::fmt::Display;

use crate::MyApp;



pub fn func() -> anyhow::Result<MyApp> {
   
    let client = BasicClient::new(
        ClientId::new("376938585937-m4str7csah18co512007sd15h7sq471e.apps.googleusercontent.com".to_string()),
        Some(ClientSecret::new("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".to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(
        "http://127.0.0.1:8080/callback".to_string(),
    )?);

  
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.courses.readonly".to_string(),  
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.rosters.readonly".to_string(),
        ))
        .url();

    println!("Opening browser for Google login...");
    let auth_str = auth_url.to_string();
   
    open::that(&auth_str)?;

let html = r#"
<!DOCTYPE html>
<html lang='en'>
<head>
<meta charset='UTF-8'>
<title>Close Window</title>
<style>
    html, body {
        height: 100%;
        margin: 0;
        display: flex;
        justify-content: center;
        align-items: center;
        font-family: Arial, sans-serif;
        background: linear-gradient(270deg, green, blue);
        background-size: 400% 400%;
        animation: gradientShift 8s ease infinite;
    }
    @keyframes gradientShift {
        0% { background-position: 0% 50%; }
        50% { background-position: 100% 50%; }
        100% { background-position: 0% 50%; }
    }
    button {
        padding: 1em 2em;
        font-size: 1.2em;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        background-color: white;
        transition: transform 0.2s, background-color 0.2s;
    }
    button:hover {
        transform: scale(1.1);
        background-color: #f0f0f0;
    }
</style>
</head>
<body>
<button onclick="window.close()">Close This Window</button>
</body>
</html>
"#;
    let server = Server::http("127.0.0.1:8080").map_err(|e| anyhow::anyhow!(e))?;
    println!("Waiting for Google redirect on http://127.0.0.1:8080/callback ...");

    let request = server.recv()?;
    let url = request.url().to_string(); 
    //let response = Response::from_string(html);
     let response = Response::from_string(html)
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..]).unwrap());
    request.respond(response)?;
   

    let query = url.split('?').nth(1).unwrap_or("");
    let params: std::collections::HashMap<_, _> = 
        form_urlencoded::parse(query.as_bytes()).into_owned().collect();

    let code = params
        .get("code")
        .expect("No code parameter found in redirect URL");
    let _state = params
        .get("state")
        .expect("No state parameter found (CSRF protection)");

    println!("Authorization code received: {}", code);


    let token = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request(http_client)
        .map_err(|err| {
            eprintln!("Token exchange failed: {:#?}", err);
            anyhow::anyhow!("token exchange failed: {:#?}", err)
        })?;

    println!("Access token: {}", token.access_token().secret());
    println!(
        "Refresh token: {:?}",
        token.refresh_token().map(|r| r.secret().to_string())
    );

    let conn = Connection::open("google_users.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS google_users (
            id INTEGER PRIMARY KEY,
            access_token TEXT NOT NULL,
            refresh_token TEXT
        )",
        [],
    )?;
    conn.execute(
        "INSERT INTO google_users (access_token, refresh_token) VALUES (?1, ?2)",
        rusqlite::params![
            token.access_token().secret(),
            token.refresh_token().map(|r| r.secret())
        ],
    )?;

    println!("Tokens stored in google_users.db");


    let access_token = token.access_token().secret().to_string();
    let client = reqwest::blocking::Client::new();
    let res = client
        .get("https://classroom.googleapis.com/v1/courses")
        .bearer_auth(access_token)
        .send()
        .map_err(|e| anyhow::anyhow!("request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().unwrap_or_else(|_| "<failed to read body>".to_string());
        eprintln!("Classroom API returned error: {}\n{}", status, body);
        return Err(anyhow::anyhow!("Classroom API error: {}", status));
    }

    let body = res.text()?;
    println!("User's Classroom courses:\n{}", body);

    Ok(MyApp {
        state: "login".to_string(),
        Info: vec![format!("{}", body)],
        texture: None,
    })
}

