use job_queue::{
    config::get_config,
    startup::build_app,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> job_queue::error::Result<()> {
    let subscriber = get_subscriber("job_queue".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config()?;
    let base_url = config.application.base_url.clone();
    let app = build_app(config).await?;
    println!("app running {}:{}", base_url, app.port());
    let application_task = tokio::spawn(app.run_until_stopped());

    tokio::select! {
        o = application_task => println!("application_task stopped: {:?}",o)
    }
    Ok(())
}
