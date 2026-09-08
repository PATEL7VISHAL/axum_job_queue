#![allow(unused)]

pub mod common;

use axum::{http, response};
use common::Result;
use reqwest::{Client, StatusCode};

#[tokio::test]
pub async fn health_check_ok() -> Result<()> {
    let app = common::test_app().await;

    let client = Client::new();
    let response = client
        .get(app.get_full_url("/health_check"))
        .send()
        .await
        .expect("failed to send reqwest");

    assert_eq!(response.status(), StatusCode::OK);

    app.cleanup().await?;
    Ok(())
}
