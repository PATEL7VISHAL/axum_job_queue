use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(format!(
                "{} enviroment is not supported. use `local` or `production`.",
                value
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub require_ssl: bool,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn get_connection_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(&self.password)
            .database(&self.database_name)
            .ssl_mode(match self.require_ssl {
                true => sqlx::postgres::PgSslMode::Require,
                false => sqlx::postgres::PgSslMode::Disable,
            })
    }
}

pub fn get_config() -> crate::error::Result<Settings> {
    let path = std::env::current_dir().expect("Failed to determin current ");
    let config_directory = path.join("configuration");
    let envirmonemt: Environment = std::env::var("APP_ENVIROMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIROMENT");

    let setting = config::Config::builder()
        .add_source(config::File::from(config_directory.join("base.yaml")))
        .add_source(config::File::from(config_directory.join(
            config_directory.join(format!("{}.yaml", envirmonemt.as_str())),
        )))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    Ok(setting.try_deserialize::<Settings>()?)
}
