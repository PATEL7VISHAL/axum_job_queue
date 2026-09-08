use sqlx::{postgres::PgPoolOptions, PgConnection, PgPool};

use crate::{config::DatabaseSettings, error::Result, store::Store};

#[derive(Clone)]
pub struct AppState {
    pg_pool: PgPool,
    hmac_secret: String,
}

fn get_db_connection_pool(db_config: DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(db_config.get_connection_options())
}

pub fn get_app_state(config: crate::config::Settings) -> Result<AppState> {
    let pg_pool = get_db_connection_pool(config.database);
    Ok(AppState {
        pg_pool,
        hmac_secret: config.application.hmac_secret,
    })
}

impl<'a> AppState {
    pub fn get_store(&'a self) -> Store<'a> {
        Store::new(&self.pg_pool)
    }
}
