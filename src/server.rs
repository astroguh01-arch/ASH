use oauth2::{
    basic::BasicClient,
    AuthUrl,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    RedirectUrl,
    Scope,
    TokenUrl,
    TokenResponse,
    reqwest::async_http_client,
};
use open;
use rusqlite::Connection;
use tiny_http::{Server, Response, Header};
use crate::env1::env;
use url::form_urlencoded;
use serde::{Deserialize};
use serde_json;
use crate::MyApp;
use crate::CourseData;



#[derive(Debug, Deserialize)]
struct CoursesResponse {
    courses: Vec<Course>,
}

#[derive(Debug, Deserialize)]
struct Course {
    id: String,
    name: String,
    room: Option<String>,
    section: Option<String>,
     #[serde(rename = "alternateLink")]
    alternate_link: Option<String>, 
}

#[derive(Debug, Deserialize)]
struct CourseworkResponse {
    #[serde(rename = "courseWork")]
    course_work: Vec<CourseWork>,
}

#[derive(Debug, Deserialize)]
struct CourseWork {
    id: String,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    id: String,
    name: Option<String>,
    email_address: Option<String>,
}
#[derive(Debug, Deserialize)]
struct StudentSubmission {
    id: String,
    course_work_id: String,
    state: Option<String>,
    assigned_grade: Option<f64>, // god help us debug this
}
use crate::ScheduleEntry;

pub async fn func() -> anyhow::Result<MyApp> {
   
    let client = BasicClient::new(
        ClientId::new(env::clientid.to_string()),
        Some(ClientSecret::new(env::jwt_secret.to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:8080/callback".to_string())?);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.courses.readonly".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.rosters.readonly".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.coursework.me.readonly".to_string(),
        ))
    
          .add_scope(Scope::new(
            "https://www.googleapis.com/auth/classroom.student-submissions.me.readonly".to_string(),
        ))
        .url();

    println!("Opening browser for Google login...");
    open::that(auth_url.to_string())?;

   
    let html = r#"
<!DOCTYPE html>
<html lang='en'>
<head><meta charset='UTF-8'><title>Close Window</title>
<style>
    html, body {height:100%;margin:0;display:flex;justify-content:center;align-items:center;font-family:Arial,sans-serif;
        background:linear-gradient(270deg,green,blue);background-size:400% 400%;animation:gradientShift 8s ease infinite;}
    @keyframes gradientShift {0%{background-position:0% 50%;}50%{background-position:100% 50%;}100%{background-position:0% 50%;}}
    button{padding:1em 2em;font-size:1.2em;border:none;border-radius:8px;cursor:pointer;background-color:white;transition:transform 0.2s,bg-color 0.2s;}
    button:hover{transform:scale(1.1);background-color:#f0f0f0;}
</style></head>
<body><button onclick="window.close()">Close This Window</button></body>
</html>
"#;

    let (code, _state) = tokio::task::spawn_blocking(move || -> anyhow::Result<(String, String)> {
        let server = Server::http("127.0.0.1:8080").map_err(|e| anyhow::anyhow!(e))?;
        println!("Waiting for Google redirect on http://127.0.0.1:8080/callback ...");
        let request = server.recv().map_err(|e| anyhow::anyhow!(e))?;
        let url = request.url().to_string();
        let response = Response::from_string(html)
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..])
            .unwrap());
        request.respond(response).map_err(|e| anyhow::anyhow!(e))?;
        let query = url.split('?').nth(1).unwrap_or("");
        let params: std::collections::HashMap<_, _> =
            form_urlencoded::parse(query.as_bytes()).into_owned().collect();
        let code = params
            .get("code")
            .ok_or_else(|| anyhow::anyhow!("No code parameter found in redirect URL"))?
            .to_string();
        let state = params
            .get("state")
            .ok_or_else(|| anyhow::anyhow!("No state parameter found (CSRF protection)"))?
            .to_string();
        println!("Authorization code received: {}", code);
        Ok((code, state))
    })
    .await??;

    
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|err| {
            eprintln!("Token exchange failed: {:#?}", err);
            anyhow::anyhow!("token exchange failed: {:#?}", err)
        })?;

    println!("Access token: {}", token.access_token().secret());
    println!(
        "Refresh token: {:?}",
        token.refresh_token().map(|r| r.secret().to_string())
    );

    let access_token_clone = token.access_token().secret().to_string();
    let http_client = reqwest::Client::new();
    
   
    let user_profile_resp = http_client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(&access_token_clone)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch user profile: {}", e))?;
    
    if !user_profile_resp.status().is_success() {
        let status = user_profile_resp.status();
        let body = user_profile_resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Failed to fetch user profile: {} {}", status, body));
    }
    
    let user_profile_body = user_profile_resp.text().await?;
    let user_profile: UserProfile = serde_json::from_str(&user_profile_body)?;
    let user_id = user_profile.id;
    println!("User ID: {}", user_id);

    {
        let at = token.access_token().secret().to_string();
        let rt = token.refresh_token().map(|r| r.secret().to_string());
        let uid = user_id.clone();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let conn = Connection::open("google_users.db")?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS google_users (
                    id INTEGER PRIMARY KEY,
                    user_id TEXT UNIQUE NOT NULL,
                    access_token TEXT NOT NULL,
                    refresh_token TEXT
                )",
                [],
            )?;
            conn.execute(
                "INSERT OR REPLACE INTO google_users (user_id, access_token, refresh_token) VALUES (?1, ?2, ?3)",
                rusqlite::params![uid, at, rt],
            )?;
            
          
            conn.execute(
                "CREATE TABLE IF NOT EXISTS schedule_entries (
                    id INTEGER PRIMARY KEY,
                    user_id TEXT NOT NULL,
                    day_of_week INTEGER NOT NULL,
                    hour INTEGER NOT NULL,
                    minute INTEGER NOT NULL,
                    course TEXT NOT NULL,
                    assignment TEXT NOT NULL,
                    FOREIGN KEY(user_id) REFERENCES google_users(user_id)
                )",
                [],
            )?;
            
            println!("Tokens and user stored in google_users.db");
            Ok(())
        })
        .await??;
    }

    let access_token = access_token_clone;

  
    let courses_body = http_client
        .get("https://classroom.googleapis.com/v1/courses")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("request failed: {}", e))?;

    if !courses_body.status().is_success() {
        let status = courses_body.status();
        let body = courses_body.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
        eprintln!("Classroom API returned error: {}\n{}", status, body);
        return Err(anyhow::anyhow!("Classroom API error: {}", status));
    }

    let body = courses_body.text().await?;
    println!("User's Classroom courses:\n{}", body);
    let parsed: CoursesResponse = serde_json::from_str(&body)?;

  
    let schedule_entries = tokio::task::spawn_blocking({
        let uid = user_id.clone();
        move || -> anyhow::Result<Vec<ScheduleEntry>> {
            let conn = Connection::open("google_users.db")?;
            let mut stmt = conn.prepare(
                "SELECT day_of_week, hour, minute, course, assignment FROM schedule_entries WHERE user_id = ? ORDER BY day_of_week, hour, minute",
            )?;
            let entries = stmt.query_map(rusqlite::params![uid], |row| {
                Ok(ScheduleEntry {
                    day_of_week: row.get(0)?,
                    hour: row.get(1)?,
                    minute: row.get(2)?,
                    course: row.get(3)?,
                    assignment: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            Ok(entries)
        }
    })
    .await??;
    
    let mut course_list: Vec<CourseData> = Vec::new();

   for course in &parsed.courses {
    let mut latest_assignment: Option<String> = None;
    let mut latest_grade: Option<f64> = None;

   
    let cw_resp = http_client
        .get(format!(
            "https://classroom.googleapis.com/v1/courses/{}/courseWork",
            course.id
        ))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("coursework request failed: {}", e))?;

    if cw_resp.status().is_success() {
        let cw_body = cw_resp.text().await?;
        if let Ok(cw_parsed) = serde_json::from_str::<CourseworkResponse>(&cw_body) {
            
           
            if let Some(first) = cw_parsed.course_work.first() {
                latest_assignment = first.title.clone().or_else(|| first.description.clone());
            }

            
            for cw in &cw_parsed.course_work {
                let sub_resp = http_client
                    .get(format!(
                        "https://classroom.googleapis.com/v1/courses/{}/courseWork/{}/studentSubmissions/me",
                        course.id,
                        cw.id
                    ))
                    .bearer_auth(&access_token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("student submission request failed: {}", e))?;

                if sub_resp.status().is_success() {
                    let sub_body = sub_resp.text().await?;
                    if let Ok(submission) = serde_json::from_str::<StudentSubmission>(&sub_body) {
                        
                        latest_grade = submission.assigned_grade;
                        break; 
                    }
                } else {
                    let status = sub_resp.status();
                    let body = sub_resp.text().await.unwrap_or_default();
                    eprintln!(
                        "Error fetching submission for {}: {}\n{}",
                        cw.title.clone().unwrap_or_default(),
                        status,
                        body
                    );
                }
            }
        }
    } else {
        let status = cw_resp.status();
        let body = cw_resp.text().await.unwrap_or_default();
        eprintln!("Error fetching coursework for {}: {}\n{}", course.name, status, body);
    }
    
    
    course_list.push(CourseData {
        name: course.name.clone(),
        latest_assignment,
        grade: latest_grade.map(|g| g.to_string()),
        alternate_link: course.alternate_link.clone(),
    });
}

    Ok(MyApp {
        state: "login".to_string(),
        courses: course_list,
        course_tabs: Vec::new(),
        texture: None,
        pending: None,
        current_user_id: user_id,
        schedule_entries,
        schedule_form_day: 0,
        schedule_form_hour: "09".to_string(),
        schedule_form_minute: "00".to_string(),
        schedule_form_course: 0,
        schedule_form_assignment: String::new(),
        rx: None,
        tx: None,
    })
}

pub fn save_schedule_entry(user_id: &str, entry: &ScheduleEntry) -> anyhow::Result<()> {
    let conn = Connection::open("google_users.db")?;
    conn.execute(
        "INSERT INTO schedule_entries (user_id, day_of_week, hour, minute, course, assignment) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![user_id, entry.day_of_week, entry.hour, entry.minute, &entry.course, &entry.assignment],
    )?;
    Ok(())
}

