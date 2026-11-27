use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, DbErr, Statement, TransactionTrait,
};
use std::{env, fmt, time::Duration};
use uuid::Uuid;

const DEFAULT_MAX_CONNECTIONS: u32 = 20;
const DEFAULT_MIN_CONNECTIONS: u32 = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A Postgres transaction scoped to a single authenticated user.
///
/// Sets both `statement_timeout` and the transaction-local `app.current_user_id`
/// setting used by Postgres Row-Level Security policies, and captures the
/// transaction's Postgres `NOW()` timestamp in a single round-trip.
pub struct UserTx {
    tx: DatabaseTransaction,
    user_id: UserId,
    now: DateTime<Utc>,
}

impl UserTx {
    pub async fn begin(db: &DatabaseConnection, user_id: UserId) -> Result<Self, DbErr> {
        let tx = db.begin().await?;
        let row = tx
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT set_config('statement_timeout', '5000', true),
                        set_config('app.current_user_id', $1, true),
                        NOW() AS now",
                vec![user_id.0.to_string().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("UserTx setup query returned no rows".to_owned()))?;
        let now: DateTime<Utc> = row.try_get("", "now")?;
        Ok(Self { tx, user_id, now })
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.now
    }

    pub fn conn(&self) -> &DatabaseTransaction {
        &self.tx
    }

    pub async fn commit(self) -> Result<(), DbErr> {
        self.tx.commit().await
    }
}

pub async fn configure_app_role_password(
    db: &DatabaseConnection,
    password: &str,
) -> Result<(), DbErr> {
    let tx = db.begin().await?;
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT set_config('app.role_password', $1, true)",
        vec![password.into()],
    ))
    .await?;
    tx.execute_unprepared(
        "DO $$
         BEGIN
             EXECUTE format(
                 'ALTER ROLE alg_trainers_app WITH LOGIN PASSWORD %L',
                 current_setting('app.role_password')
             );
         END
         $$;",
    )
    .await?;
    tx.commit().await
}

pub async fn connect() -> Result<DatabaseConnection, DbErr> {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be configured explicitly");
    let max_connections = env_u32("DATABASE_MAX_CONNECTIONS", DEFAULT_MAX_CONNECTIONS);
    let min_connections = env_u32("DATABASE_MIN_CONNECTIONS", DEFAULT_MIN_CONNECTIONS);
    if max_connections == 0 || min_connections > max_connections {
        panic!(
            "DATABASE_MIN_CONNECTIONS must be no greater than DATABASE_MAX_CONNECTIONS, \
             which must be greater than zero"
        );
    }
    let mut options = ConnectOptions::new(database_url);
    options
        .max_connections(max_connections)
        .min_connections(min_connections)
        .connect_timeout(env_duration_secs("DATABASE_CONNECT_TIMEOUT_SECS", 5))
        .acquire_timeout(env_duration_secs("DATABASE_ACQUIRE_TIMEOUT_SECS", 5))
        .idle_timeout(env_duration_secs("DATABASE_IDLE_TIMEOUT_SECS", 300))
        .max_lifetime(env_duration_secs("DATABASE_MAX_LIFETIME_SECS", 1800));
    Database::connect(options).await
}

fn env_u32(name: &str, default: u32) -> u32 {
    parse_env(name, default, str::parse)
}

fn env_duration_secs(name: &str, default: u64) -> Duration {
    Duration::from_secs(parse_env(name, default, str::parse))
}

fn parse_env<T, E>(name: &str, default: T, parser: impl FnOnce(&str) -> Result<T, E>) -> T
where
    T: std::fmt::Display,
    E: std::fmt::Display,
{
    let Ok(raw) = env::var(name) else {
        return default;
    };
    parser(&raw).unwrap_or_else(|error| {
        panic!("{name} must be numeric, got {raw:?}: {error}");
    })
}
