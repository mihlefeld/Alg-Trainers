mod catalog;
mod sync;

use alg_trainers_migration::Migrator;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use rocket::{
    http::{Cookie, CookieJar, Status},
    response::Redirect,
    serde::json::Json,
    State,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    wca_base: String,
    wca_client_id: String,
    wca_client_secret: String,
    wca_redirect_url: String,
}

#[derive(Serialize)]
struct MeResponse {
    authenticated: bool,
    email: Option<String>,
    wca_id: Option<String>,
}

#[derive(Deserialize)]
struct WcaTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct WcaUser {
    email: Option<String>,
    wca_id: Option<String>,
}

#[derive(Deserialize)]
struct WcaMeEnvelope {
    me: WcaUser,
}

fn cookie_user(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get_private("alg_trainers_session")
        .map(|c| c.value().to_owned())
}

async fn find_user(db: &DatabaseConnection, email: &str) -> Option<uuid::Uuid> {
    db.query_one(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM users WHERE email = $1",
        vec![email.into()],
    ))
    .await
    .ok()
    .flatten()
    .and_then(|row| row.try_get("", "id").ok())
}

fn random_state() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

#[rocket::get("/api/me")]
async fn me(cookies: &CookieJar<'_>, state: &State<AppState>) -> Json<MeResponse> {
    let email = cookie_user(cookies);
    let (email, wca_id) = match email {
        Some(email) => {
            let row = state
                .db
                .query_one(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::Postgres,
                    "SELECT email, wca_id FROM users WHERE email = $1",
                    vec![email.into()],
                ))
                .await
                .ok()
                .flatten();
            match row {
                Some(row) => (
                    row.try_get::<String>("", "email").ok(),
                    row.try_get::<Option<String>>("", "wca_id").ok().flatten(),
                ),
                None => (None, None),
            }
        }
        None => (None, None),
    };
    Json(MeResponse {
        authenticated: email.is_some(),
        email,
        wca_id,
    })
}

#[rocket::get("/api/auth/wca")]
fn wca_login(cookies: &CookieJar<'_>, state: &State<AppState>) -> Redirect {
    let state_value = random_state();
    let mut cookie = Cookie::new("wca_oauth_state", state_value.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(rocket::http::SameSite::Lax);
    cookies.add_private(cookie);
    let url = format!(
        "{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        state.wca_base,
        urlencoding::encode(&state.wca_client_id),
        urlencoding::encode(&state.wca_redirect_url),
        urlencoding::encode("public email"),
        urlencoding::encode(&state_value),
    );
    Redirect::to(url)
}

#[rocket::get("/api/auth/wca/callback?<code>&<state>")]
async fn wca_callback(
    cookies: &CookieJar<'_>,
    app_state: &State<AppState>,
    code: Option<String>,
    state: Option<String>,
) -> Result<Redirect, Status> {
    let code = code.ok_or(Status::BadRequest)?;
    let returned_state = state.ok_or(Status::BadRequest)?;
    let saved = cookies
        .get_private("wca_oauth_state")
        .map(|c| c.value().to_owned());
    if saved.as_deref() != Some(returned_state.as_str()) {
        return Err(Status::Unauthorized);
    }
    cookies.remove_private(Cookie::from("wca_oauth_state"));

    let client = reqwest::Client::new();
    let token: WcaTokenResponse = client
        .post(format!("{}/oauth/token", app_state.wca_base))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", app_state.wca_client_id.as_str()),
            ("client_secret", app_state.wca_client_secret.as_str()),
            ("redirect_uri", app_state.wca_redirect_url.as_str()),
            ("code", code.as_str()),
        ])
        .send()
        .await
        .map_err(|_| Status::BadGateway)?
        .error_for_status()
        .map_err(|_| Status::BadGateway)?
        .json()
        .await
        .map_err(|_| Status::BadGateway)?;

    let envelope: WcaMeEnvelope = client
        .get(format!("{}/api/v0/me", app_state.wca_base))
        .bearer_auth(token.access_token)
        .send()
        .await
        .map_err(|_| Status::BadGateway)?
        .error_for_status()
        .map_err(|_| Status::BadGateway)?
        .json()
        .await
        .map_err(|_| Status::BadGateway)?;

    let email = envelope.me.email.ok_or(Status::Unauthorized)?;
    let wca_id = envelope.me.wca_id;
    if let Some(id) = find_user(&app_state.db, &email).await {
        app_state
            .db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "UPDATE users SET wca_id = $1 WHERE id = $2",
                vec![wca_id.clone().into(), id.into()],
            ))
            .await
            .map_err(|_| Status::InternalServerError)?;
    } else {
        app_state
            .db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "INSERT INTO users (id, email, wca_id, created_at) VALUES ($1, $2, $3, $4)",
                vec![
                    uuid::Uuid::new_v4().into(),
                    email.clone().into(),
                    wca_id.clone().into(),
                    Utc::now().into(),
                ],
            ))
            .await
            .map_err(|_| Status::InternalServerError)?;
    }
    let mut cookie = Cookie::new("alg_trainers_session", email.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(rocket::http::SameSite::Lax);
    cookie.set_max_age(rocket::time::Duration::days(30));
    cookies.add_private(cookie);
    Ok(Redirect::to("/"))
}

#[rocket::post("/api/auth/logout")]
fn logout(cookies: &CookieJar<'_>) -> Status {
    cookies.remove_private(Cookie::from("alg_trainers_session"));
    Status::Ok
}

#[rocket::get("/api/trainers")]
async fn trainers(state: &State<AppState>) -> Json<Vec<serde_json::Value>> {
    Json(
        catalog::list(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|trainer| serde_json::to_value(trainer).unwrap_or_default())
            .collect(),
    )
}

#[rocket::get("/api/trainers/<trainer_id>")]
async fn trainer(
    state: &State<AppState>,
    trainer_id: &str,
) -> Result<Json<catalog::TrainerBundle>, Status> {
    catalog::load(&state.db, trainer_id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .map(Json)
        .ok_or(Status::NotFound)
}

#[rocket::get("/api/sync")]
async fn sync_get(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Json<sync::SyncBundle>, Status> {
    let email = cookie_user(cookies).ok_or(Status::Unauthorized)?;
    let user_id = find_user(&state.db, &email)
        .await
        .ok_or(Status::Unauthorized)?;
    sync::load(&state.db, user_id)
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[rocket::put("/api/sync", data = "<bundle>")]
async fn sync_put(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    bundle: Json<sync::SyncBundle>,
) -> Result<Json<sync::SyncBundle>, Status> {
    let email = cookie_user(cookies).ok_or(Status::Unauthorized)?;
    let user_id = find_user(&state.db, &email)
        .await
        .ok_or(Status::Unauthorized)?;
    sync::save(&state.db, user_id, bundle.into_inner())
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[rocket::get("/")]
async fn index() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("frontend/dist/index.html")
        .await
        .ok()
}

#[rocket::get("/<path..>", rank = 20)]
async fn catch_all(path: std::path::PathBuf) -> Option<rocket::fs::NamedFile> {
    if let Ok(stripped) = path.strip_prefix("catalog") {
        if stripped
            .extension()
            .is_some_and(|extension| extension == "png")
        {
            if let Ok(file) =
                rocket::fs::NamedFile::open(std::path::Path::new("catalog").join(stripped)).await
            {
                return Some(file);
            }
        }
    }
    if path.starts_with("style")
        || matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("icon.png" | "manifest.json" | "serviceworker.js")
        )
    {
        if let Ok(file) = rocket::fs::NamedFile::open(path.clone()).await {
            return Some(file);
        }
    }
    let frontend = std::path::PathBuf::from("frontend/dist").join(&path);
    if let Ok(file) = rocket::fs::NamedFile::open(frontend).await {
        return Some(file);
    }
    if path.extension().is_some() {
        return None;
    }
    rocket::fs::NamedFile::open("frontend/dist/index.html")
        .await
        .ok()
}

#[rocket::launch]
async fn rocket() -> _ {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://alg_trainers:alg_trainers@psql:5432/alg_trainers".to_owned()
    });
    let db = Database::connect(&database_url)
        .await
        .expect("Unable to connect to PostgreSQL");
    Migrator::up(&db, None)
        .await
        .expect("Database migrations failed");
    let catalog_root = env::var("CATALOG_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    catalog::seed(&db, &catalog_root)
        .await
        .expect("Unable to seed trainer catalog");

    let state = AppState {
        db,
        wca_base: env::var("WCA_BASE_URL")
            .unwrap_or_else(|_| "https://www.worldcubeassociation.org".to_owned()),
        wca_client_id: env::var("WCA_CLIENT_ID").unwrap_or_default(),
        wca_client_secret: env::var("WCA_CLIENT_SECRET").unwrap_or_default(),
        wca_redirect_url: env::var("WCA_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:8000/api/auth/wca/callback".to_owned()),
    };

    rocket::build().manage(state).mount(
        "/",
        rocket::routes![
            index,
            me,
            wca_login,
            wca_callback,
            logout,
            trainers,
            trainer,
            sync_get,
            sync_put,
            catch_all
        ],
    )
}
