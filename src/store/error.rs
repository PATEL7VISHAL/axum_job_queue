use derive_more::From;

pub(super) type Result<T> = core::result::Result<T, StoreError>;

#[derive(Debug, From)]
pub enum StoreError {
    #[from]
    DatabaseError(sqlx::Error),
}
