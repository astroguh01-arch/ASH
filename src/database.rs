use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl, reqwest::http_client,
};
use rusqlite::Connection;
use tiny_http::{Server, Response};
use url::form_urlencoded;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    // =========================
    // 1. Configure OAuth2 Client
    // =========================
    let client = BasicClient::new(
        ClientId::new("YOUR_CLIENT_ID".to_string()),
        Some(ClientSecret::new("YOUR_CLIENT_SECRET".to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(
        "http://127.0.0.1:8080/callback".to_string(),
    )?);

    // =========================
    // 2. Generate auth URL
    // =========================
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.courses.readonly".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.rosters.readonly".to_string(),
        ))
        .url();

    println!("Opening browser for Google login...");
    // Open in default browser
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "start", &auth_url.to_string()]).spawn()?;
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&auth_url.to_string()).spawn()?;
    } else {
        Command::new("xdg-open").arg(&auth_url.to_string()).spawn()?;
    }

    // =========================
    // 3. Start local server to catch redirect
    // =========================
    let server = Server::http("127.0.0.1:8080")?;
    println!("Waiting for Google redirect on http://127.0.0.1:8080/callback ...");

    let request = server.recv()?;
    let url = request.url(); // e.g., "/callback?code=...&state=..."
    let response = Response::from_string("You can close this window now!");
    request.respond(response)?;

    // Parse query params
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

    // =========================
    // 4. Exchange code for tokens
    // =========================
    let token = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request(http_client)?;

    println!("Access token: {}", token.access_token().secret());
    println!(
        "Refresh token: {:?}",
        token.refresh_token().map(|r| r.secret().to_string())
    );

    // =========================
    // 5. Store tokens in SQLite
    // =========================
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
        &[
            &token.access_token().secret(),
            &token.refresh_token().map(|r| r.secret()),
        ],
    )?;

    println!("Tokens stored in google_users.db");

    // =========================
    // 6. Example: fetch Classroom courses
    // =========================
    let access_token = token.access_token().secret();
    let client = reqwest::blocking::Client::new();
    let res = client
        .get("https://classroom.googleapis.com/v1/courses")
        .bearer_auth(access_token)
        .send()?;

    let body = res.text()?;
    println!("User's Classroom courses:\n{}", body);

    Ok(())
}
