use axum::response::IntoResponse;
use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Config(config::ConfigError),

    #[from]
    Io(std::io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let res = axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        return res;
    }
}
