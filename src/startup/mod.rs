use std::fmt::format;

use crate::{config::Settings, error::Result, startup::state::get_app_state};
use axum::{
    routing::{get, post, IntoMakeService},
    serve::{self, Serve},
    Router,
};
use tokio::{net::TcpListener, sync::oneshot};

mod state;
use crate::api;
pub use state::AppState;

type Server = serve::Serve<TcpListener, IntoMakeService<Router>, Router>;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<()> {
        Ok(self.server.await?)
    }

    pub async fn run_with_graceful_shutdown(self, signal: oneshot::Receiver<()>) -> Result<()> {
        Ok(self
            .server
            .with_graceful_shutdown(async move {
                signal.await.unwrap();
            })
            .await?)
    }
}

pub async fn build_app(config: Settings) -> Result<Application> {
    let base_url = config.application.base_url.clone();
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app_state = get_app_state(config)?;
    let server = run(listener, port, base_url, app_state).await?;
    Ok(Application { server, port })
}

pub async fn run(
    listener: TcpListener,
    port: u16,
    base_url: String,
    app_state: AppState,
) -> Result<Server> {
    let server = axum::serve::serve(
        listener,
        Router::new()
            .route("/health_check", get(api::health_check::router))
            .with_state(app_state.clone())
            .into_make_service(),
    );
    Ok(server)
}
