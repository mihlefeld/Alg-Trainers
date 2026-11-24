use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::{env, time::Duration};

const DEFAULT_MAX_CONNECTIONS: u32 = 20;
const DEFAULT_MIN_CONNECTIONS: u32 = 2;

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
