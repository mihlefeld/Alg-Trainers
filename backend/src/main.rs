mod catalog;
mod database;
#[allow(dead_code)]
mod entity;
mod pwa;
mod sync;

use alg_trainers_migration::Migrator;
use alg_trainers_shared::MeResponse;
use chrono::Utc;
use database::{UserId, UserTx};
use rand::{distributions::Alphanumeric, Rng};
use rocket::{
    http::{ContentType, Cookie, CookieJar, Header, Status},
    request::{self, FromRequest, Request},
    response::Redirect,
    serde::json::Json,
    tokio::sync::OnceCell,
    Responder, State,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

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
    service_worker_js: Arc<String>,
    offline_bundle_cache: Arc<OnceCell<Arc<String>>>,
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
struct OfflineTrainersBundle(String, ContentType, Header<'static>);

#[derive(Responder)]
struct ServiceWorkerScript(String, ContentType, Header<'static>, Header<'static>);

#[derive(Responder)]
struct ManifestResponse(String, ContentType, Header<'static>);

#[derive(Responder)]
struct PrivateMe(Json<MeResponse>, Header<'static>);

#[derive(Responder)]
struct PrivateSync(Json<sync::SyncBundle>, Header<'static>);

#[derive(Deserialize)]
struct WcaTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct WcaUser {
    id: Option<i64>,
    email: Option<String>,
    wca_id: Option<String>,
}

#[derive(Deserialize)]
struct WcaMeEnvelope {
    me: WcaUser,
}

struct AuthenticatedUser {
    id: UserId,
    email: String,
    wca_id: Option<String>,
}

struct OptionalUser(Option<AuthenticatedUser>);

async fn resolve_session_user(
    db: &DatabaseConnection,
    cookies: &CookieJar<'_>,
) -> Result<Option<AuthenticatedUser>, DbErr> {
    let Some(raw) = cookies
        .get_private("alg_trainers_session")
        .map(|c| c.value().to_owned())
    else {
        return Ok(None);
    };
    let user = if let Ok(user_id) = Uuid::parse_str(&raw) {
        entity::user::Entity::find_by_id(user_id).one(db).await?
    } else {
        // Legacy fallback for pre-migration cookies that stored email.
        find_user_by_email(db, &raw).await?
    };
    Ok(user.map(|model| AuthenticatedUser {
        id: UserId(model.id),
        email: model.email,
        wca_id: model.wca_id,
    }))
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(state) = request.rocket().state::<AppState>() else {
            return request::Outcome::Error((Status::InternalServerError, ()));
        };
        match resolve_session_user(&state.db, request.cookies()).await {
            Ok(user) => request::Outcome::Success(OptionalUser(user)),
            Err(error) => {
                tracing::error!(%error, "failed to load authenticated user");
                request::Outcome::Error((Status::InternalServerError, ()))
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match OptionalUser::from_request(request).await {
            request::Outcome::Success(OptionalUser(Some(user))) => request::Outcome::Success(user),
            request::Outcome::Success(OptionalUser(None)) => {
                request::Outcome::Error((Status::Unauthorized, ()))
            }
            request::Outcome::Error(error) => request::Outcome::Error(error),
            request::Outcome::Forward(status) => request::Outcome::Forward(status),
        }
    }
}

async fn find_user_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Option<entity::user::Model>, DbErr> {
    entity::user::Entity::find()
        .filter(entity::user::Column::Email.eq(email))
        .one(db)
        .await
}

async fn find_user_by_wca_user_id(
    db: &DatabaseConnection,
    wca_user_id: i64,
) -> Result<Option<entity::user::Model>, DbErr> {
    entity::user::Entity::find()
        .filter(entity::user::Column::WcaUserId.eq(wca_user_id))
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
async fn me(user: OptionalUser, _rate_limit: ApiRateLimit) -> PrivateMe {
    let (email, wca_id) = match user.0 {
        Some(user) => (Some(user.email), user.wca_id),
        None => (None, None),
    };
    PrivateMe(
        Json(MeResponse {
            authenticated: email.is_some(),
            email,
            wca_id,
        }),
        Header::new("Cache-Control", "no-store"),
    )
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
    let wca_user_id = envelope.me.id;
    let wca_id = envelope.me.wca_id;
    let existing_user = match wca_user_id {
        Some(id) => match find_user_by_wca_user_id(&app_state.db, id)
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to look up WCA user by id");
                Status::InternalServerError
            })? {
            Some(user) => Some(user),
            None => find_user_by_email(&app_state.db, &email)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "failed to look up WCA user by email");
                    Status::InternalServerError
                })?
                // Only link by email for legacy rows that have not yet been bound to a WCA user ID.
                .filter(|user| user.wca_user_id.is_none()),
        },
        None => find_user_by_email(&app_state.db, &email)
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to look up WCA user by email");
                Status::InternalServerError
            })?,
    };
    let user_id = if let Some(user) = existing_user {
        let user_id = user.id;
        let email_changed = user.email != email;
        if email_changed {
            if let Some(conflicting) = find_user_by_email(&app_state.db, &email)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "failed to check WCA email uniqueness");
                    Status::InternalServerError
                })?
            {
                if conflicting.id != user_id {
                    tracing::warn!(
                        user_id = %user_id,
                        conflicting_user_id = %conflicting.id,
                        "WCA email change conflicts with another account"
                    );
                    return Err(Status::Conflict);
                }
            }
        }
        let mut active: entity::user::ActiveModel = user.into();
        if email_changed {
            active.email = Set(email);
        }
        if let Some(id) = wca_user_id {
            active.wca_user_id = Set(Some(id));
        }
        active.wca_id = Set(wca_id);
        active.update(&app_state.db).await.map_err(|error| {
            tracing::error!(%error, "failed to update WCA user");
            Status::InternalServerError
        })?;
        user_id
    } else {
        if find_user_by_email(&app_state.db, &email)
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to check WCA email availability");
                Status::InternalServerError
            })?
            .is_some()
        {
            tracing::warn!("WCA login email already belongs to another WCA user ID");
            return Err(Status::Conflict);
        }
        let user_id = Uuid::new_v4();
        entity::user::ActiveModel {
            id: Set(user_id),
            email: Set(email),
            wca_user_id: Set(wca_user_id),
            wca_id: Set(wca_id),
            created_at: Set(Utc::now()),
        }
        .insert(&app_state.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to create WCA user");
            Status::InternalServerError
        })?;
        user_id
    };
    let mut cookie = Cookie::new("alg_trainers_session", user_id.to_string());
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

#[rocket::get("/api/offline/trainers")]
async fn offline_trainers(
    state: &State<AppState>,
    _rate_limit: ApiRateLimit,
) -> Result<OfflineTrainersBundle, Status> {
    let payload = state
        .offline_bundle_cache
        .get_or_try_init(|| async {
            let bundles = catalog::load_all(&state.db).await.map_err(|error| {
                tracing::error!(%error, "failed to load offline trainers bundle");
                Status::InternalServerError
            })?;
            let json = serde_json::to_string(&bundles).map_err(|error| {
                tracing::error!(%error, "failed to serialize offline trainers bundle");
                Status::InternalServerError
            })?;
            Ok::<Arc<String>, Status>(Arc::new(json))
        })
        .await?;
    Ok(OfflineTrainersBundle(
        payload.as_ref().clone(),
        ContentType::JSON,
        Header::new(
            "Cache-Control",
            "public, max-age=3600, stale-while-revalidate=86400",
        ),
    ))
}

#[rocket::get("/sw.js")]
fn service_worker(state: &State<AppState>) -> ServiceWorkerScript {
    ServiceWorkerScript(
        state.service_worker_js.as_ref().clone(),
        ContentType::JavaScript,
        Header::new("Cache-Control", "no-cache, no-store, must-revalidate"),
        Header::new("Service-Worker-Allowed", "/"),
    )
}

#[rocket::get("/manifest.json")]
async fn manifest() -> Result<ManifestResponse, Status> {
    let content = rocket::tokio::fs::read_to_string("manifest.json")
        .await
        .map_err(|_| Status::NotFound)?;
    Ok(ManifestResponse(
        content,
        ContentType::new("application", "manifest+json"),
        Header::new("Cache-Control", "no-cache"),
    ))
}

#[rocket::get("/icon.png")]
async fn icon_png() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("icon.png").await.ok()
}

#[rocket::get("/mobile.png")]
async fn mobile_png() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("mobile.png").await.ok()
}

#[rocket::get("/api/sync?<since>&<before_datetime>&<before_id>")]
async fn sync_get(
    user: AuthenticatedUser,
    state: &State<AppState>,
    _rate_limit: ApiRateLimit,
    since: Option<String>,
    before_datetime: Option<String>,
    before_id: Option<String>,
) -> Result<PrivateSync, Status> {
    let since = match since {
        Some(value) => Some(
            chrono::DateTime::parse_from_rfc3339(&value)
                .map_err(|_| Status::BadRequest)?
                .with_timezone(&Utc),
        ),
        None => None,
    };
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
    let tx = UserTx::begin(&state.db, user.id).await.map_err(|error| {
        tracing::error!(%error, user_id = %user.id, "failed to begin user transaction");
        Status::InternalServerError
    })?;
    let bundle = sync::load_page(&tx, since, cursor).await.map_err(|error| {
        tracing::error!(%error, user_id = %user.id, "failed to load sync data");
        Status::InternalServerError
    })?;
    tx.commit().await.map_err(|error| {
        tracing::error!(%error, user_id = %user.id, "failed to commit sync read transaction");
        Status::InternalServerError
    })?;
    Ok(PrivateSync(
        Json(bundle),
        Header::new("Cache-Control", "no-store"),
    ))
}

#[rocket::put("/api/sync", data = "<bundle>")]
async fn sync_put(
    user: AuthenticatedUser,
    state: &State<AppState>,
    bundle: Json<sync::SyncBundle>,
    _rate_limit: ApiRateLimit,
) -> Result<PrivateSync, Status> {
    let validated = sync::ValidatedSyncBundle::try_from(bundle.into_inner()).map_err(|error| {
        tracing::warn!(%error, user_id = %user.id, "rejected invalid sync payload");
        Status::UnprocessableEntity
    })?;
    let tx = UserTx::begin(&state.db, user.id).await.map_err(|error| {
        tracing::error!(%error, user_id = %user.id, "failed to begin user transaction");
        Status::InternalServerError
    })?;
    let response = sync::save(&tx, validated).await.map_err(|error| {
        tracing::error!(%error, user_id = %user.id, "failed to save sync payload");
        Status::InternalServerError
    })?;
    tx.commit().await.map_err(|error| {
        tracing::error!(%error, user_id = %user.id, "failed to commit sync write transaction");
        Status::InternalServerError
    })?;
    Ok(PrivateSync(
        Json(response),
        Header::new("Cache-Control", "no-store"),
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
    let catalog_root = env::var("CATALOG_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| "static".into());
    if run_migrations {
        Migrator::up(&db, None)
            .await
            .expect("Database migrations failed");
        if let Some(app_password) = env::var("DATABASE_APP_PASSWORD")
            .ok()
            .filter(|value| !value.is_empty())
        {
            database::configure_app_role_password(&db, &app_password)
                .await
                .expect("Unable to configure alg_trainers_app role password");
        }
        catalog::seed(&db, &catalog_root)
            .await
            .expect("Unable to seed trainer catalog");
    }
    if migrate_only {
        std::process::exit(0);
    }

    let trainer_ids: Vec<String> = catalog::list(&db)
        .await
        .expect("Unable to list trainers for PWA service worker")
        .into_iter()
        .map(|trainer| trainer.trainer_id)
        .collect();
    let pwa_version_override = env::var("PWA_VERSION").ok();
    let pwa_assets = pwa::build_pwa_assets(
        std::path::Path::new("frontend/dist"),
        std::path::Path::new("style"),
        &catalog_root,
        &trainer_ids,
        pwa_version_override.as_deref(),
    );
    tracing::info!(pwa_version = %pwa_assets.version, trainers = trainer_ids.len(), "prepared PWA service worker");

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
        service_worker_js: Arc::new(pwa_assets.service_worker_js),
        offline_bundle_cache: Arc::new(OnceCell::new()),
    };
    let rate_limiter = RateLimiter {
        buckets: Mutex::new(HashMap::new()),
    };

    rocket::build().manage(state).manage(rate_limiter).mount(
        "/",
        rocket::routes![
            index,
            service_worker,
            manifest,
            icon_png,
            mobile_png,
            me,
            wca_login,
            wca_callback,
            logout,
            trainers,
            trainer,
            offline_trainers,
            sync_get,
            sync_put,
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
    use super::{secure_cookie_required, MeResponse};

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

    #[test]
    fn me_response_serializes_wca_id_as_camel_case() {
        let value = serde_json::to_value(MeResponse {
            authenticated: true,
            email: Some("user@example.com".to_owned()),
            wca_id: Some("2020TEST01".to_owned()),
        })
        .expect("serialize MeResponse");
        assert_eq!(value["wcaId"], "2020TEST01");
        assert!(value.get("wca_id").is_none());
    }
}
