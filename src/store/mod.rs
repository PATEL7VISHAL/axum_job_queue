mod error;

use sqlx::PgPool;

pub struct Store<'a> {
    pool: &'a PgPool,
}
