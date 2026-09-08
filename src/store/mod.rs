mod error;

use sqlx::PgPool;

pub struct Store<'a> {
    pool: &'a PgPool,
}

impl<'a> Store<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}
