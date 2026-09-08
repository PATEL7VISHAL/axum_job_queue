#![allow(unused)]

use std::{fmt::format, sync::LazyLock};

use axum::routing::connect;
use job_queue::{
    config::{get_config, DatabaseSettings},
    startup::build_app,
    store::Store,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{database, postgres::PgQueryResult, Connection, Executor, PgConnection, PgPool};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber_name = "test".into();
    let default_fliter_level = "debug".into();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_fliter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_fliter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

async fn get_maintance_settings(port: u16, host: String) -> PgConnection {
    let maintenance_settings: DatabaseSettings = {
        DatabaseSettings {
            database_name: "postgres".into(),
            username: "postgres".into(),
            password: std::env::var("POSTGRES_DEV_USER_PWD")
                .unwrap_or("postgres".into())
                .into(),
            port,
            host,
            require_ssl: false,
        }
    };

    let mut connection = PgConnection::connect_with(&maintenance_settings.get_connection_options())
        .await
        .expect("failed to connect with postgres");

    connection
}

async fn config_database(config: DatabaseSettings) -> PgPool {
    let mut connection = get_maintance_settings(config.port, config.host.clone()).await;

    connection
        .execute(
            format!(
                r#"CREATE DATABASE "{}" OWNER {};"#,
                config.database_name, config.username
            )
            .as_str(),
        )
        .await
        .expect("FAILED TO CREATE DATABASE FOR TESTING");

    let connection_pool = PgPool::connect_with(config.get_connection_options())
        .await
        .expect("FAILED TO CONNECT DATABASE");

    //NOTE: migration
    connection_pool
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    db_pool: PgPool,
    server_shutdown_sender: Option<tokio::sync::oneshot::Sender<()>>,
}

impl<'a> TestApp {
    pub fn get_full_url(&self, req: &str) -> String {
        format!("{}:{}{}", self.address, self.port, req)
    }
    pub fn get_pool(&self) -> &PgPool {
        &self.db_pool
    }
    pub fn get_store(&'a self) -> Store<'a> {
        Store::new(&self.db_pool)
    }

    pub async fn cleanup(self) -> Result<()> {
        self.db_pool.close().await;

        let connection_options = self.db_pool.connect_options();
        let port = connection_options.get_port();
        let host = connection_options.get_host().to_string();
        let database_name = connection_options.get_database().unwrap().to_string();
        let mut maintance_connection = get_maintance_settings(port, host).await;
        maintance_connection
            .execute(format!(r#"DROP DATABASE IF EXISTS "{}" (FORCE);"#, database_name).as_str())
            .await
            .expect("FAILED TO RELEASE TMP TESTING DATABASE");

        Ok(())
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if let Some(sender) = self.server_shutdown_sender.take() {
            println!("shutdown the server");
            sender.send(()).expect("failed to send shutdown single")
        }
    }
}

pub async fn test_app() -> TestApp {
    LazyLock::force(&TRACING);
    let config = {
        let mut c = get_config().expect("FAILED TO GET COFNIG");
        c.application.port = 0;
        c.database.database_name = format!("_test_job_queue{}", uuid::Uuid::new_v4());
        c
    };

    let db_pool = config_database(config.database.clone()).await;
    let address = config.application.base_url.clone();
    let (shutdown_sx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let app = build_app(config).await.expect("FAILED TO BUILD THE SERVER");
    let test_app = TestApp {
        address,
        port: app.port(),
        server_shutdown_sender: Some(shutdown_sx),
        db_pool,
    };

    let _ = tokio::spawn(app.run_with_graceful_shutdown(shutdown_rx));
    test_app
}
