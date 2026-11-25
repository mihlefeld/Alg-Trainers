mod catalog;
mod database;
#[allow(dead_code)]
mod entity;
mod sync;

use alg_trainers_migration::Migrator;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use rocket::{
    http::{Cookie, CookieJar, Header, Status},
    request::{self, FromRequest, Request},
    response::Redirect,
    serde::json::Json,
    Responder, State,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs,
    net::IpAddr,
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};

const DEFAULT_SECRET_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    http: reqwest::Client,
    wca_base: String,
    wca_client_id: String,
    wca_client_secret: String,
    wca_redirect_url: String,
    cookie_secure: bool,
}

struct RateLimiter {
    buckets: Mutex<HashMap<(IpAddr, &'static str), RateBucket>>,
}

struct RateBucket {
    started: Instant,
    requests: u32,
}

struct ApiRateLimit;

impl RateLimiter {
    fn allow(&self, request: &Request<'_>) -> bool {
        let ip = request
            .client_ip()
            .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1]));
        let (bucket_name, limit) = match request.uri().path().as_str() {
            path if path.starts_with("/api/auth") => ("auth", 30),
            "/api/sync" => ("sync", 30),
            path if path.starts_with("/api/trainers/") => ("catalog-detail", 20),
            _ => ("api", 120),
        };
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate-limit lock poisoned");
        if buckets.len() > 10_000 {
            buckets
                .retain(|_, bucket| now.duration_since(bucket.started) < Duration::from_secs(60));
        }
        let bucket = buckets.entry((ip, bucket_name)).or_insert(RateBucket {
            started: now,
            requests: 0,
        });
        if now.duration_since(bucket.started) >= Duration::from_secs(60) {
            bucket.started = now;
            bucket.requests = 0;
        }
        bucket.requests += 1;
        bucket.requests <= limit
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiRateLimit {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(limiter) = request.rocket().state::<RateLimiter>() else {
            return request::Outcome::Error((Status::InternalServerError, ()));
        };
        if limiter.allow(request) {
            request::Outcome::Success(Self)
        } else {
            request::Outcome::Error((Status::TooManyRequests, ()))
        }
    }
}

#[derive(Responder)]
struct CachedTrainers(Json<Vec<serde_json::Value>>, Header<'static>);

#[derive(Responder)]
struct CachedTrainer(Json<catalog::TrainerBundle>, Header<'static>);

#[derive(Responder)]
struct PrivateMe(Json<MeResponse>, Header<'static>);

#[derive(Responder)]
struct PrivateSync(Json<sync::SyncBundle>, Header<'static>);
#[derive(Responder)]
struct VersionJson(Json<serde_json::Value>, Header<'static>);

#[derive(Responder)]
struct PwaManifest(Json<serde_json::Value>, Header<'static>);

#[derive(Responder)]
struct ServiceWorker(rocket::fs::NamedFile, Header<'static>, Header<'static>);

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

async fn find_user(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Option<entity::user::Model>, DbErr> {
    entity::user::Entity::find()
        .filter(entity::user::Column::Email.eq(email))
        .one(db)
        .await
}

fn random_state() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn secure_cookie_required(redirect_url: &str, configured: Option<&str>) -> bool {
    let uses_https = redirect_url.starts_with("https://");
    match configured.and_then(|value| value.parse::<bool>().ok()) {
        Some(true) if uses_https => true,
        Some(true) => {
            tracing::warn!(
                redirect_url,
                "COOKIE_SECURE=true is incompatible with an HTTP callback; disabling it for this deployment"
            );
            false
        }
        Some(false) => false,
        None => uses_https,
    }
}

#[rocket::get("/api/me")]
async fn me(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    _rate_limit: ApiRateLimit,
) -> Result<PrivateMe, Status> {
    let email = cookie_user(cookies);
    let (email, wca_id) = match email {
        Some(email) => {
            let user = find_user(&state.db, &email).await.map_err(|error| {
                tracing::error!(%error, "failed to load authenticated user");
                Status::InternalServerError
            })?;
            match user {
                Some(user) => (Some(user.email), user.wca_id),
                None => (None, None),
            }
        }
        None => (None, None),
    };
    Ok(PrivateMe(
        Json(MeResponse {
            authenticated: email.is_some(),
            email,
            wca_id,
        }),
        Header::new("Cache-Control", "no-store"),
    ))
}

#[rocket::get("/api/auth/wca")]
fn wca_login(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    _rate_limit: ApiRateLimit,
) -> Redirect {
    let state_value = random_state();
    let mut cookie = Cookie::new("wca_oauth_state", state_value.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(rocket::http::SameSite::Lax);
    cookie.set_secure(state.cookie_secure);
    cookie.set_max_age(rocket::time::Duration::minutes(10));
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
    _rate_limit: ApiRateLimit,
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

    let token: WcaTokenResponse = app_state
        .http
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

    let envelope: WcaMeEnvelope = app_state
        .http
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
    if let Some(user) = find_user(&app_state.db, &email).await.map_err(|error| {
        tracing::error!(%error, "failed to look up WCA user");
        Status::InternalServerError
    })? {
        let mut active: entity::user::ActiveModel = user.into();
        active.wca_id = Set(wca_id.clone());
        active.update(&app_state.db).await.map_err(|error| {
            tracing::error!(%error, "failed to update WCA user");
            Status::InternalServerError
        })?;
    } else {
        entity::user::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            email: Set(email.clone()),
            wca_id: Set(wca_id.clone()),
            created_at: Set(Utc::now()),
        }
        .insert(&app_state.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to create WCA user");
            Status::InternalServerError
        })?;
    }
    let mut cookie = Cookie::new("alg_trainers_session", email.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(rocket::http::SameSite::Lax);
    cookie.set_secure(app_state.cookie_secure);
    cookie.set_max_age(rocket::time::Duration::days(30));
    cookies.add_private(cookie);
    Ok(Redirect::to("/"))
}

#[rocket::post("/api/auth/logout")]
fn logout(cookies: &CookieJar<'_>, _rate_limit: ApiRateLimit) -> Status {
    cookies.remove_private(Cookie::from("alg_trainers_session"));
    Status::Ok
}

#[rocket::get("/api/trainers")]
async fn trainers(
    state: &State<AppState>,
    _rate_limit: ApiRateLimit,
) -> Result<CachedTrainers, Status> {
    let trainers = catalog::list(&state.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to list trainer catalog");
            Status::InternalServerError
        })?
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Status::InternalServerError)?;
    Ok(CachedTrainers(
        Json(trainers),
        Header::new(
            "Cache-Control",
            "public, max-age=3600, stale-while-revalidate=86400",
        ),
    ))
}

#[rocket::get("/api/trainers/<trainer_id>")]
async fn trainer(
    state: &State<AppState>,
    trainer_id: &str,
    _rate_limit: ApiRateLimit,
) -> Result<CachedTrainer, Status> {
    catalog::load(&state.db, trainer_id)
        .await
        .map_err(|error| {
            tracing::error!(%error, %trainer_id, "failed to load trainer catalog entry");
            Status::InternalServerError
        })?
        .map(|trainer| {
            CachedTrainer(
                Json(trainer),
                Header::new(
                    "Cache-Control",
                    "public, max-age=3600, stale-while-revalidate=86400",
                ),
            )
        })
        .ok_or(Status::NotFound)
}

#[rocket::get("/api/sync?<before_datetime>&<before_id>")]
async fn sync_get(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    _rate_limit: ApiRateLimit,
    before_datetime: Option<String>,
    before_id: Option<String>,
) -> Result<PrivateSync, Status> {
    let email = cookie_user(cookies).ok_or(Status::Unauthorized)?;
    let user = find_user(&state.db, &email)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to load sync user");
            Status::InternalServerError
        })?
        .ok_or(Status::Unauthorized)?;
    let cursor = match (before_datetime, before_id) {
        (None, None) => None,
        (Some(datetime), Some(id)) => Some(sync::ResultCursor {
            datetime: chrono::DateTime::parse_from_rfc3339(&datetime)
                .map_err(|_| Status::BadRequest)?
                .with_timezone(&Utc),
            id: id.parse().map_err(|_| Status::BadRequest)?,
        }),
        _ => return Err(Status::BadRequest),
    };
    sync::load_page(&state.db, user.id, cursor)
        .await
        .map(|bundle| PrivateSync(Json(bundle), Header::new("Cache-Control", "no-store")))
        .map_err(|error| {
            tracing::error!(%error, user_id = %user.id, "failed to load sync data");
            Status::InternalServerError
        })
}

#[rocket::put("/api/sync", data = "<bundle>")]
async fn sync_put(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    bundle: Json<sync::SyncBundle>,
    _rate_limit: ApiRateLimit,
) -> Result<PrivateSync, Status> {
    let email = cookie_user(cookies).ok_or(Status::Unauthorized)?;
    let user = find_user(&state.db, &email)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to load sync user");
            Status::InternalServerError
        })?
        .ok_or(Status::Unauthorized)?;
    sync::save(&state.db, user.id, bundle.into_inner())
        .await
        .map(|bundle| PrivateSync(Json(bundle), Header::new("Cache-Control", "no-store")))
        .map_err(|error| match error {
            sync::SaveError::Validation(error) => {
                tracing::warn!(%error, user_id = %user.id, "rejected invalid sync payload");
                Status::UnprocessableEntity
            }
            sync::SaveError::Database(error) => {
                tracing::error!(%error, user_id = %user.id, "failed to save sync payload");
                Status::InternalServerError
            }
        })
}

#[derive(Serialize)]
struct OfflineAsset {
    url: String,
}

#[derive(Serialize)]
struct OfflineManifest {
    version: String,
    catalog_version: String,
    assets: Vec<OfflineAsset>,
}

fn collect_dist_assets(root: &Path, prefix: &str, assets: &mut Vec<OfflineAsset>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dist_assets(&path, prefix, assets);
        } else if let Ok(relative) = path.strip_prefix(prefix) {
            if let Some(url) = relative.to_str() {
                assets.push(OfflineAsset {
                    url: format!("/{}", url.replace('\\', "/")),
                });
            }
        }
    }
}

#[rocket::get("/offline-manifest.json")]
fn offline_manifest() -> Json<OfflineManifest> {
    let version = env::var("APP_VERSION")
        .or_else(|_| env::var("GIT_SHA"))
        .unwrap_or_else(|_| "dev".to_owned());
    let catalog_version = env::var("CATALOG_VERSION").unwrap_or_else(|_| "catalog-v1".to_owned());
    let mut assets = vec![
        OfflineAsset { url: "/".to_owned() },
        OfflineAsset { url: "/manifest.json".to_owned() },
        OfflineAsset { url: "/icon.png".to_owned() },
        OfflineAsset { url: "/mobile.png".to_owned() },
        OfflineAsset { url: "/style/main.css".to_owned() },
        OfflineAsset { url: "/style/selection.css".to_owned() },
        OfflineAsset { url: "/style/timer.css".to_owned() },
        OfflineAsset { url: "/style/settings.css".to_owned() },
        OfflineAsset { url: "/style/hint.css".to_owned() },
        OfflineAsset { url: "/style/algsheet.css".to_owned() },
        OfflineAsset { url: "/style/fonts/titillium-web-latin-400-normal.woff2".to_owned() },
        OfflineAsset { url: "/style/fonts/titillium-web-latin-400-italic.woff2".to_owned() },
        OfflineAsset { url: "/style/fonts/titillium-web-latin-600-normal.woff2".to_owned() },
        OfflineAsset { url: "/style/fonts/titillium-web-latin-700-normal.woff2".to_owned() },
        OfflineAsset { url: "/style/fonts/material-symbols-outlined-latin-400-normal.woff2".to_owned() },
        OfflineAsset { url: "/api/trainers".to_owned() },
    ];
    collect_dist_assets(Path::new("frontend/dist"), "frontend/dist", &mut assets);
    if let Ok(entries) = fs::read_dir("static") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(id) = path.file_name().and_then(|value| value.to_str()) {
                    assets.push(OfflineAsset {
                        url: format!("/api/trainers/{id}"),
                    });
                    let icon = path.join("icon.png");
                    if icon.exists() {
                        assets.push(OfflineAsset {
                            url: format!("/catalog/{id}/icon.png"),
                        });
                    }
                }
            }
        }
    }
    Json(OfflineManifest {
        version,
        catalog_version,
        assets,
    })
}

#[rocket::get("/manifest.json")]
fn manifest_json() -> PwaManifest {
    PwaManifest(
        Json(serde_json::json!({
            "name": "ALLG Trainer",
            "short_name": "ALLG",
            "id": "/",
            "start_url": "/",
            "scope": "/",
            "description": "Alg trainer for all types of algs!",
            "display": "standalone",
            "background_color": "#353535",
            "theme_color": "#353535",
            "icons": [
                {"src": "/icon.png", "sizes": "512x512", "type": "image/png", "purpose": "any"},
                {"src": "/mobile.png", "sizes": "512x512", "type": "image/png", "purpose": "maskable"}
            ]
        })),
        Header::new("Cache-Control", "public, max-age=3600"),
    )
}

#[rocket::get("/version.json")]
fn version_json() -> VersionJson {
    let version = env::var("APP_VERSION")
        .or_else(|_| env::var("GIT_SHA"))
        .unwrap_or_else(|_| "dev".to_owned());
    let catalog_version = env::var("CATALOG_VERSION").unwrap_or_else(|_| "catalog-v1".to_owned());
    VersionJson(
        Json(serde_json::json!({
            "version": version,
            "build": env::var("GIT_SHA").unwrap_or_else(|_| "dev".to_owned()),
            "catalogVersion": catalog_version,
            "manifest": "offline-manifest.json"
        })),
        Header::new("Cache-Control", "no-store"),
    )
}

#[rocket::get("/service-worker.js")]
async fn service_worker() -> Option<ServiceWorker> {
    let file = rocket::fs::NamedFile::open("service-worker.js")
        .await
        .ok()
        .or_else(|| None);
    if let Some(file) = file {
        return Some(ServiceWorker(
            file,
            Header::new("Service-Worker-Allowed", "/"),
            Header::new("Cache-Control", "no-cache"),
        ));
    }
    rocket::fs::NamedFile::open("frontend/dist/service-worker.js")
        .await
        .ok()
        .map(|file| ServiceWorker(
            file,
            Header::new("Service-Worker-Allowed", "/"),
            Header::new("Cache-Control", "no-cache"),
        ))
}

#[rocket::get("/")]
async fn index() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("frontend/dist/index.html")
        .await
        .ok()
}

#[rocket::get("/<path..>", rank = 20)]
async fn catch_all(path: std::path::PathBuf) -> Option<rocket::fs::NamedFile> {
    if path == std::path::Path::new("manifest.json") {
        if let Ok(file) = rocket::fs::NamedFile::open("manifest.json").await {
            return Some(file);
        }
        if let Ok(file) = rocket::fs::NamedFile::open("frontend/dist/manifest.json").await {
            return Some(file);
        }
    }
    if matches!(path.to_str(), Some("icon.png" | "mobile.png")) {
        if let Ok(file) = rocket::fs::NamedFile::open(path.clone()).await {
            return Some(file);
        }
        if let Ok(file) = rocket::fs::NamedFile::open(format!("frontend/dist/{}", path.display())).await {
            return Some(file);
        }
    }
    if path == std::path::Path::new("service-worker.js") {
        if let Ok(file) = rocket::fs::NamedFile::open("service-worker.js").await {
            return Some(file);
        }
        if let Ok(file) = rocket::fs::NamedFile::open("frontend/dist/service-worker.js").await {
            return Some(file);
        }
    }
    if let Ok(stripped) = path.strip_prefix("catalog") {
        if stripped
            .extension()
            .is_some_and(|extension| extension == "png")
        {
            if let Ok(file) =
                rocket::fs::NamedFile::open(std::path::Path::new("static").join(stripped)).await
            {
                return Some(file);
            }
        }
    }
    if path.starts_with("style") {
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
    let db = database::connect()
        .await
        .expect("Unable to connect to PostgreSQL");
    let migrate_only = env_bool("DATABASE_MIGRATE_ONLY").unwrap_or(false);
    let run_migrations = migrate_only || env_bool("DATABASE_RUN_MIGRATIONS").unwrap_or(true);
    if run_migrations {
        Migrator::up(&db, None)
            .await
            .expect("Database migrations failed");
        let catalog_root = env::var("CATALOG_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| "static".into());
        catalog::seed(&db, &catalog_root)
            .await
            .expect("Unable to seed trainer catalog");
    }
    if migrate_only {
        std::process::exit(0);
    }

    let secret_key = env::var("ROCKET_SECRET_KEY").expect("ROCKET_SECRET_KEY must be configured");
    if secret_key == DEFAULT_SECRET_KEY
        || secret_key.len() < 32
        || secret_key.contains("change-me")
        || secret_key.contains("replace-with")
    {
        panic!("ROCKET_SECRET_KEY must be replaced with a high-entropy secret");
    }
    let wca_redirect_url = env::var("WCA_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8000/api/auth/wca/callback".to_owned());
    let configured_cookie_secure = env::var("COOKIE_SECURE").ok();
    let cookie_secure =
        secure_cookie_required(&wca_redirect_url, configured_cookie_secure.as_deref());
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Unable to build HTTP client");
    let state = AppState {
        db,
        http,
        wca_base: env::var("WCA_BASE_URL")
            .unwrap_or_else(|_| "https://www.worldcubeassociation.org".to_owned()),
        wca_client_id: env::var("WCA_CLIENT_ID").unwrap_or_default(),
        wca_client_secret: env::var("WCA_CLIENT_SECRET").unwrap_or_default(),
        wca_redirect_url,
        cookie_secure,
    };
    let rate_limiter = RateLimiter {
        buckets: Mutex::new(HashMap::new()),
    };

    rocket::build().manage(state).manage(rate_limiter).mount(
        "/",
        rocket::routes![
            index,
            service_worker,
            me,
            wca_login,
            wca_callback,
            logout,
            trainers,
            trainer,
            sync_get,
            sync_put,
            manifest_json,
            offline_manifest,
            version_json,
            catch_all
        ],
    )
}

fn env_bool(name: &str) -> Result<bool, &'static str> {
    match env::var(name).ok().as_deref() {
        None | Some("") => Err("not configured"),
        Some("1" | "true" | "TRUE") => Ok(true),
        Some("0" | "false" | "FALSE") => Ok(false),
        Some(_) => panic!("{name} must be true, false, 1, or 0"),
    }
}

#[cfg(test)]
mod tests {
    use super::secure_cookie_required;

    #[test]
    fn http_callback_cannot_set_secure_oauth_cookie() {
        assert!(!secure_cookie_required(
            "http://localhost:8000/api/auth/wca/callback",
            Some("true")
        ));
        assert!(!secure_cookie_required(
            "http://localhost:8000/api/auth/wca/callback",
            None
        ));
    }

    #[test]
    fn https_callback_defaults_to_secure_cookie() {
        assert!(secure_cookie_required(
            "https://trainer.example/api/auth/wca/callback",
            None
        ));
    }
}
